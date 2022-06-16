use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates timer queues and timer queue handlers
#[allow(clippy::too_many_lines)]
pub fn codegen(app: &App, analysis: &Analysis, _extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    if !app.monotonics.is_empty() {
        // Generate the marker counter used to track for `cancel` and `reschedule`
        let tq_marker = util::timer_queue_marker_ident();
        items.push(quote!(
            // #[doc = #doc]
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            static #tq_marker: rtic::RacyCell<u32> = rtic::RacyCell::new(0);
        ));

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

            // For future use
            // let doc = "Tasks that can be scheduled".to_string();
            items.push(quote!(
                // #[doc = #doc]
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                pub enum #t {
                    #(#variants,)*
                }
            ));
        }
    }

    for (_, monotonic) in &app.monotonics {
        let monotonic_name = monotonic.ident.to_string();
        let tq = util::tq_ident(&monotonic_name);
        let t = util::schedule_t_ident();
        let mono_type = &monotonic.ty;
        let m_ident = util::monotonic_ident(&monotonic_name);

        // Static variables and resource proxy
        {
            // For future use
            // let doc = &format!("Timer queue for {}", monotonic_name);
            let cap: usize = app
                .software_tasks
                .iter()
                .map(|(_name, task)| task.args.capacity as usize)
                .sum();
            let n = util::capacity_literal(cap);
            let tq_ty = quote!(rtic::export::TimerQueue<#mono_type, #t, #n>);

            // For future use
            // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
            items.push(quote!(
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #[allow(non_upper_case_globals)]
                static #tq: rtic::RacyCell<#tq_ty> =
                    rtic::RacyCell::new(rtic::export::TimerQueue(rtic::export::SortedLinkedList::new_u16()));
            ));

            let mono = util::monotonic_ident(&monotonic_name);
            // For future use
            // let doc = &format!("Storage for {}", monotonic_name);

            items.push(quote!(
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #[allow(non_upper_case_globals)]
                static #mono: rtic::RacyCell<Option<#mono_type>> = rtic::RacyCell::new(None);
            ));
        }

        // Timer queue handler
        {
            let enum_ = util::interrupt_ident();
            let rt_err = util::rt_err_ident();

            let arms = app
                .software_tasks
                .iter()
                .map(|(name, task)| {
                    let cfgs = &task.cfgs;
                    let priority = task.args.priority;
                    let rq = util::rq_ident(priority);
                    let rqt = util::spawn_t_ident(priority);

                    // The interrupt that runs the task dispatcher
                    let interrupt = &analysis.interrupts.get(&priority).expect("RTIC-ICE: interrupt not found").0;

                    let pend = {
                        quote!(
                            rtic::pend(#rt_err::#enum_::#interrupt);
                        )
                    };

                    quote!(
                        #(#cfgs)*
                        #t::#name => {
                            rtic::export::interrupt::free(|_| (&mut *#rq.get_mut()).split().0.enqueue_unchecked((#rqt::#name, index)));

                            #pend
                        }
                    )
                })
                .collect::<Vec<_>>();

            let bound_interrupt = &monotonic.args.binds;
            let disable_isr = if &*bound_interrupt.to_string() == "SysTick" {
                quote!(core::mem::transmute::<_, rtic::export::SYST>(()).disable_interrupt())
            } else {
                quote!(rtic::export::NVIC::mask(#rt_err::#enum_::#bound_interrupt))
            };

            items.push(quote!(
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe fn #bound_interrupt() {
                    while let Some((task, index)) = rtic::export::interrupt::free(|_|
                        if let Some(mono) = (&mut *#m_ident.get_mut()).as_mut() {
                            (&mut *#tq.get_mut()).dequeue(|| #disable_isr, mono)
                        } else {
                            // We can only use the timer queue if `init` has returned, and it
                            // writes the `Some(monotonic)` we are accessing here.
                            core::hint::unreachable_unchecked()
                        })
                    {
                        match task {
                            #(#arms)*
                        }
                    }

                    rtic::export::interrupt::free(|_| if let Some(mono) = (&mut *#m_ident.get_mut()).as_mut() {
                        mono.on_interrupt();
                    });
                }
            ));
        }
    }

    items
}
