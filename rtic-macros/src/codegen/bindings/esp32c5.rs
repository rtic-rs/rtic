#[cfg(feature = "riscv-esp32c5")]
pub use esp32c5::*;

#[cfg(feature = "riscv-esp32c5")]
#[allow(clippy::module_inception)]
mod esp32c5 {
    use crate::{
        analyze::Analysis as CodegenAnalysis,
        codegen::util,
        syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
    };
    use core::ops::Range;
    use proc_macro2::{Span, TokenStream as TokenStream2};
    use quote::quote;
    use std::collections::HashSet;
    use syn::{parse, Attribute, Ident};

    const HANDLER_EXPORT_NAME_PREFIX: &str = "__rtic_interrupt_handler_";

    // CLINT has lower 16 reserved interrupts : 0..=15
    // esp-hal reserves interrupts : 16..=(16+PRIO_MAX)
    // esp-hal::DirectBindableCpuInterrupt::Interrupt0 = CpuInterrupt::Interrupt24
    const DIRECT_BINDABLE_CPU_INTERRUPTS: Range<usize> = 0_usize..24;

    #[allow(clippy::too_many_arguments)]
    pub fn impl_mutex(
        _app: &App,
        _analysis: &CodegenAnalysis,
        cfgs: &[Attribute],
        resources_prefix: bool,
        name: &Ident,
        ty: &TokenStream2,
        ceiling: u8,
        ptr: &TokenStream2,
    ) -> TokenStream2 {
        let path = if resources_prefix {
            quote!(shared_resources::#name)
        } else {
            quote!(#name)
        };
        quote!(
            #(#cfgs)*
            impl<'a> rtic::Mutex for #path<'a> {
                type T = #ty;

                #[inline(always)]
                fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {
                    /// Priority ceiling
                    const CEILING: u8 = #ceiling;
                    unsafe {
                        rtic::export::lock(
                            #ptr,
                            CEILING,
                            f,
                        )
                    }
                }
            }
        )
    }

    pub fn interrupt_ident() -> Ident {
        let span = Span::call_site();
        Ident::new("Interrupt", span)
    }

    pub fn interrupt_mod(app: &App) -> TokenStream2 {
        let device = &app.args.device;
        let interrupt = interrupt_ident();
        quote!(#device::peripherals::#interrupt)
    }

    pub fn extra_assertions(_: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![quote!(let a = "extra_assertions";)]
    }

    pub fn pre_init_preprocessing(app: &mut App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
        // Peripherals and core in user app are accessed via esp_hal::init
        app.args.core = false;
        app.args.peripherals = false;

        Ok(())
    }

    pub fn pre_init_checks(app: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
        let mut stmts = vec![];
        // check that all dispatchers exists in the `Interrupt` enumeration regardless of whether
        // they are used or not
        let rt_err = util::rt_err_ident();

        for name in app.args.dispatchers.keys() {
            stmts.push(quote!(let _ = #rt_err::peripherals::Interrupt::#name;));
        }
        stmts
    }

    pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        let mut stmts = vec![];
        let rt_err = util::rt_err_ident();

        let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));
        // Unmask interrupts and set their priorities
        for ((&priority, name), curr_cpu_id) in interrupt_ids
            .chain(
                app.hardware_tasks
                    .values()
                    .map(|task| (&task.args.priority, &task.args.binds)),
            )
            .zip(DIRECT_BINDABLE_CPU_INTERRUPTS)
        {
            let es = format!(
                "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
            );
            let es_zero = format!("Priority {priority} used by interrupt vector '{name}' is less than supported by hardware");
            // Compile time assert that this priority is supported by the device
            stmts.push(quote!(
                const _: () =  if (#rt_err::interrupt::Priority::max() as usize) <= #priority as usize { ::core::panic!(#es); };
                const _: () =  if (#rt_err::interrupt::Priority::min() as usize) > #priority as usize { ::core::panic!(#es_zero);};
            ));

            let prio_variant = Ident::new(&format!("Priority{priority}"), Span::call_site());
            let bindable_interrupt_enum =
                Ident::new(&format!("Interrupt{curr_cpu_id}"), Span::call_site());
            let trap_start = Ident::new(
                &format!("_start_{HANDLER_EXPORT_NAME_PREFIX}{name}_trap"),
                Span::call_site(),
            );
            stmts.push(quote!(
                unsafe extern "C" {fn #trap_start();}

                rtic::export::enable(
                    #rt_err::peripherals::Interrupt::#name,
                    #rt_err::interrupt::Priority::#prio_variant,
                    #rt_err::interrupt::DirectBindableCpuInterrupt::#bindable_interrupt_enum,
                    #trap_start
                );
            ));
        }
        stmts
    }

    pub fn architecture_specific_analysis(
        app: &App,
        _analysis: &SyntaxAnalysis,
    ) -> parse::Result<()> {
        //check if the dispatchers are supported
        for name in app.args.dispatchers.keys() {
            let name_s = name.to_string();
            match &*name_s {
                "FROM_CPU_INTR0" | "FROM_CPU_INTR1" | "FROM_CPU_INTR2" | "FROM_CPU_INTR3" => {}

                _ => {
                    return Err(parse::Error::new(
                        name.span(),
                        "Only FROM_CPU_INTRX are supported as dispatchers",
                    ));
                }
            }
        }

        // Check that there are enough external interrupts to dispatch the software tasks and the timer
        // queue handler
        let mut first = None;
        let priorities = app
            .software_tasks
            .iter()
            .map(|(name, task)| {
                first = Some(name);
                task.args.priority
            })
            .filter(|prio| *prio > 0)
            .collect::<HashSet<_>>();

        let need = priorities.len();
        let given = app.args.dispatchers.len();
        if need > given {
            let s = {
                format!(
                    "not enough interrupts to dispatch \
                        all software tasks (need: {need}; given: {given})"
                )
            };

            // If not enough tasks and first still is None, may cause
            // "custom attribute panicked" due to unwrap on None
            return Err(parse::Error::new(first.unwrap().span(), s));
        }
        Ok(())
    }

    pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        vec![]
    }

    pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        vec![]
    }

    pub fn async_entry(
        _app: &App,
        _analysis: &CodegenAnalysis,
        dispatcher_name: Ident,
    ) -> Vec<TokenStream2> {
        let mut stmts = vec![];
        stmts.push(quote!(
            rtic::export::unpend(rtic::export::Interrupt::#dispatcher_name); //simulate cortex-m behavior by unpending the interrupt on entry.
        ));
        stmts
    }

    pub fn async_prio_limit(_app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        let max = if let Some(max) = analysis.max_async_prio {
            quote!(#max)
        } else {
            quote!(u8::MAX) // No limit
        };

        vec![quote!(
            /// Holds the maximum priority level for use by async HAL drivers.
            #[no_mangle]
            static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8 = #max;
        )]
    }

    pub fn handler_config(
        app: &App,
        analysis: &CodegenAnalysis,
        dispatcher_name: Ident,
    ) -> Vec<TokenStream2> {
        let rt_err = util::rt_err_ident();
        let mut stmts = vec![];

        let interrupt_ids = analysis.interrupts.iter().map(|(_, (id, _))| id);
        for name in interrupt_ids.chain(app.hardware_tasks.values().map(|task| &task.args.binds)) {
            if *name == dispatcher_name {
                let core_interrupt_impl = Ident::new(
                    &format!("{HANDLER_EXPORT_NAME_PREFIX}{name}"),
                    Span::call_site(),
                );

                stmts.push(quote!(
                    #[rtic::export::riscv_rt::core_interrupt(#core_interrupt_impl)]
                    #[#rt_err::ram]
                ));
            }
        }

        stmts
    }

    pub fn extra_modules(app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
        let mut stmts = vec![];

        let rt_err = util::rt_err_ident();

        stmts.push(quote!(
            use rtic::export::riscv_rt;
        ));
        for (name, bindable_it) in app
            .args
            .dispatchers
            .keys()
            .chain(app.hardware_tasks.values().map(|task| &task.args.binds))
            .zip(DIRECT_BINDABLE_CPU_INTERRUPTS)
        {
            let core_interrupt_impl = Ident::new(
                &format!("{HANDLER_EXPORT_NAME_PREFIX}{name}"),
                Span::call_site(),
            );
            let it_variant = Ident::new(&format!("Interrupt{bindable_it}"), Span::call_site());
            stmts.push(quote!(

                #[derive(Copy, Clone)]
                struct #core_interrupt_impl;

                unsafe impl rtic::export::riscv_rt::InterruptNumber for #core_interrupt_impl {
                    const MAX_INTERRUPT_NUMBER: usize = 47;

                    fn number(self) -> usize {
                        #rt_err::interrupt::DirectBindableCpuInterrupt::#it_variant as _
                    }

                    fn from_number(_value: usize) -> rtic::export::riscv_rt::result::Result<Self> {
                        Ok(Self)
                    }
                }

                unsafe impl rtic::export::riscv_rt::CoreInterruptNumber for #core_interrupt_impl {}

            ));
        }

        stmts
    }
}
