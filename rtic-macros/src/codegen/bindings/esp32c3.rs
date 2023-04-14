#[cfg(feature = "riscv-esp32c3")]
pub use esp32c3::*;

#[cfg(feature = "riscv-esp32c3")]
mod esp32c3{
    use crate::{
        analyze::Analysis as CodegenAnalysis,
        syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
        codegen::util,
    };
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::{parse, Attribute, Ident};
    use std::collections::HashSet;

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

    pub fn extra_assertions(_: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![]
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
        let max_prio:usize = 15; //unfortunately this is not part of pac, but we know that max prio is 15.
        let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));
        // Unmask interrupts and set their priorities
        for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().filter_map(|task| {
            Some((&task.args.priority, &task.args.binds))
        })){
            let es = format!(
                "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
            );
            // Compile time assert that this priority is supported by the device
            stmts.push(quote!(
                const _: () =  if (#max_prio) <= #priority as usize { ::core::panic!(#es); };
            ));    
            //hal enables interrupt and sets prio simultaneously
            stmts.push(quote!(
                rtic::export::hal_interrupt::enable(
                    #rt_err::Interrupt::#name, //interrupt struct
                    rtic::export::int_to_prio(#priority) //interrupt priority object            
                );
            ));
        }
        stmts
    }


    pub fn architecture_specific_analysis(app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
        //check if the dispatchers are supported
        for name in app.args.dispatchers.keys() {
            let name_s = name.to_string();
            match &*name_s {
                "FROM_CPU_INTR0" | "FROM_CPU_INTR1" | "FROM_CPU_INTR2" | "FROM_CPU_INTR3" => {}

                _ => {
                    return Err(parse::Error::new(
                    name.span(),
                    "Only FROM_CPU_INTRX are supported as dispatchers",
                ));}
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

    pub fn async_prio_limit(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        let max = if let Some(max) = analysis.max_async_prio {
            quote!(#max)
        } else {
            // No limit
            let device = &app.args.device;
            quote!(1 << #device::NVIC_PRIO_BITS)
        };
    
        vec![quote!(
            /// Holds the maximum priority level for use by async HAL drivers.
            #[no_mangle]
            static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8 = #max;
        )]
    }
}