#[cfg(feature = "riscv-esp32c6")]
pub use esp32c6::*;

#[cfg(feature = "riscv-esp32c6")]
mod esp32c6 {
    use crate::{
        analyze::Analysis as CodegenAnalysis,
        codegen::util,
        syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
    };
    use proc_macro2::{Span, TokenStream as TokenStream2};
    use quote::quote;
    use std::collections::HashSet;
    use syn::{parse, Attribute, Ident};

    // Section 1.6.2 technical reference manual specifies which interrupts can be configured.
    const EXTERNAL_INTERRUPTS: [u8; 28] = [
        1, 2, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
        28, 29, 30, 31,
    ];

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
        quote!(#device::#interrupt)
    }

    pub fn extra_assertions(_: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![]
    }

    pub fn pre_init_preprocessing(_app: &mut App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
        Ok(())
    }

    pub fn pre_init_checks(app: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
        let mut stmts = vec![];
        // check that all dispatchers exists in the `Interrupt` enumeration regardless of whether
        // they are used or not
        let rt_err = util::rt_err_ident();

        for name in app.args.dispatchers.keys() {
            stmts.push(quote!(let _ = #rt_err::Interrupt::#name;));
        }
        stmts
    }

    pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        let mut stmts = vec![];
        let rt_err = util::rt_err_ident();
        let max_prio: usize = 15; //unfortunately this is not part of pac, but we know that max prio is 15.
        let min_prio: usize = 1;
        let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));
        // Unmask interrupts and set their priorities
        for ((&priority, name), curr_cpu_id) in interrupt_ids
            .chain(
                app.hardware_tasks
                    .values()
                    .filter_map(|task| Some((&task.args.priority, &task.args.binds))),
            )
            .zip(EXTERNAL_INTERRUPTS)
        {
            let es = format!(
                "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
            );
            let es_zero = format!("Priority {priority} used by interrupt vector '{name}' is less than supported by hardware");
            // Compile time assert that this priority is supported by the device
            stmts.push(quote!(
                const _: () =  if (#max_prio) <= #priority as usize { ::core::panic!(#es); };
                const _: () =  if (#min_prio) > #priority as usize { ::core::panic!(#es_zero);};
            ));
            stmts.push(quote!(
                rtic::export::enable(
                    #rt_err::Interrupt::#name,
                    #priority,
                    #curr_cpu_id,
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

    pub fn check_stack_overflow_before_init(
        _app: &App,
        _analysis: &CodegenAnalysis,
    ) -> Vec<TokenStream2> {
        vec![quote!(
            // Check for stack overflow using symbols from `risc-v-rt`.
            extern "C" {
                pub static _stack_start: u32;
                pub static _bss_end: u32;
            }

            let stack_start = &_stack_start as *const _ as u32;
            let ebss = &_bss_end as *const _ as u32;

            if stack_start > ebss {
                // No flip-link usage, check the SP for overflow.
                if rtic::export::read_sp() <= ebss {
                    panic!("Stack overflow after allocating executors");
                }
            }
        )]
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
        let mut stmts = vec![];
        let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));
        for ((_, name), curr_cpu_id) in interrupt_ids
            .chain(
                app.hardware_tasks
                    .values()
                    .filter_map(|task| Some((&task.args.priority, &task.args.binds))),
            )
            .zip(EXTERNAL_INTERRUPTS)
        {
            // interrupt1...interrupt19 are already defined in esp_hal
            if curr_cpu_id > 19 {
                if *name == dispatcher_name {
                    let ret = &("interrupt".to_owned() + &curr_cpu_id.to_string());
                    stmts.push(quote!(#[export_name = #ret]));
                }
            }
        }

        stmts
    }

    pub fn extra_modules(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![]
    }
}
