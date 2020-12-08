use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates timer queues and timer queue handlers
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    if !app.monotonics.is_empty() {
        let t = util::schedule_t_ident();

        // Enumeration of `schedule`-able tasks
        {
            let variants = app
                .software_tasks
                .iter()
                .map(|(name, task)| {
                    let cfgs = &task.cfgs;

                    quote!(
                        #(#cfgs)*
                        #name
                    )
                })
                .collect::<Vec<_>>();

            let doc = "Tasks that can be scheduled".to_string();
            items.push(quote!(
                #[doc = #doc]
                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                enum #t {
                    #(#variants,)*
                }
            ));
        }
    }

    for (_, monotonic) in &app.monotonics {
        let monotonic_name = monotonic.ident.to_string();
        let tq = util::tq_ident(&monotonic_name);
        let t = util::schedule_t_ident();
        let m = &monotonic.ident;

        // Static variables and resource proxy
        {
            let doc = &format!("Timer queue for {}", monotonic_name);
            let cap = app
                .software_tasks
                .iter()
                .map(|(_name, task)| task.args.capacity)
                .sum();
            let n = util::capacity_typenum(cap, false);
            let tq_ty = quote!(rtic::export::TimerQueue<#m, #t, #n>);

            items.push(quote!(
                #[doc = #doc]
                static mut #tq: #tq_ty = rtic::export::TimerQueue(
                    rtic::export::BinaryHeap(
                        rtic::export::iBinaryHeap::new()
                    )
                );
            ));
        }

        // Timer queue handler
        {
            let arms = app
                .software_tasks
                .iter()
                .map(|(name, task)| {
                    let cfgs = &task.cfgs;
                    let priority = task.args.priority;
                    let rq = util::rq_ident(priority);
                    let rqt = util::spawn_t_ident(priority);
                    let enum_ = util::interrupt_ident();

                    // The interrupt that runs the task dispatcher
                    let interrupt = &analysis.interrupts.get(&priority).expect("RTIC-ICE: interrupt not found").0;

                    let pend = {
                        quote!(
                            rtic::pend(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml::#enum_::#interrupt);
                        )
                    };

                    quote!(
                        #(#cfgs)*
                        #t::#name => {
                            rtic::export::interrupt::free(|_| #rq.split().0.enqueue_unchecked((#rqt::#name, index)));

                            #pend
                        }
                    )
                })
                .collect::<Vec<_>>();

            let bound_interrupt = &monotonic.args.binds;
            items.push(quote!(
                #[no_mangle]
                unsafe fn #bound_interrupt() {
                    use rtic::Mutex as _;

                    while let Some((task, index)) = rtic::export::interrupt::free(|_| #tq.dequeue())
                    {
                        match task {
                            #(#arms)*
                        }
                    }
                }
            ));
        }
    }

    items
}
