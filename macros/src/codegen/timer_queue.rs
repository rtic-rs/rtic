use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates timer queues and timer queue handlers
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    for (&sender, timer_queue) in &analysis.timer_queues {
        let cfg_sender = util::cfg_core(sender, app.args.cores);
        let t = util::schedule_t_ident(sender);

        // Enumeration of `schedule`-able tasks
        {
            let variants = timer_queue
                .tasks
                .iter()
                .map(|name| {
                    let cfgs = &app.software_tasks[name].cfgs;

                    quote!(
                        #(#cfgs)*
                        #name
                    )
                })
                .collect::<Vec<_>>();

            let doc = format!("Tasks that can be scheduled from core #{}", sender);
            items.push(quote!(
                #cfg_sender
                #[doc = #doc]
                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                enum #t {
                    #(#variants,)*
                }
            ));
        }

        let tq = util::tq_ident(sender);

        // Static variable and resource proxy
        {
            let doc = format!("Core #{} timer queue", sender);
            let m = extra.monotonic();
            let n = util::capacity_typenum(timer_queue.capacity, false);
            let tq_ty = quote!(rtfm::export::TimerQueue<#m, #t, #n>);

            let section = util::link_section("bss", sender);
            items.push(quote!(
                #cfg_sender
                #[doc = #doc]
                #section
                static mut #tq: #tq_ty = rtfm::export::TimerQueue(
                    rtfm::export::BinaryHeap(
                        rtfm::export::iBinaryHeap::new()
                    )
                );

                #cfg_sender
                struct #tq<'a> {
                    priority: &'a rtfm::export::Priority,
                }
            ));

            items.push(util::impl_mutex(
                extra,
                &[],
                cfg_sender.as_ref(),
                false,
                &tq,
                tq_ty,
                timer_queue.ceiling,
                quote!(&mut #tq),
            ));
        }

        // Timer queue handler
        {
            let device = extra.device;
            let arms = timer_queue
                .tasks
                .iter()
                .map(|name| {
                    let task = &app.software_tasks[name];

                    let cfgs = &task.cfgs;
                    let priority = task.args.priority;
                    let receiver = task.args.core;
                    let rq = util::rq_ident(receiver, priority, sender);
                    let rqt = util::spawn_t_ident(receiver, priority, sender);
                    let enum_ = util::interrupt_ident(receiver, app.args.cores);
                    let interrupt = &analysis.interrupts[&receiver][&priority];

                    let pend = if sender != receiver {
                        quote!(
                            #device::xpend(#receiver, #device::#enum_::#interrupt);
                        )
                    } else {
                        quote!(
                            rtfm::pend(#device::#enum_::#interrupt);
                        )
                    };

                    quote!(
                        #(#cfgs)*
                        #t::#name => {
                            (#rq { priority: &rtfm::export::Priority::new(PRIORITY) }).lock(|rq| {
                                rq.split().0.enqueue_unchecked((#rqt::#name, index))
                            });

                            #pend
                        }
                    )
                })
                .collect::<Vec<_>>();

            let priority = timer_queue.priority;
            let sys_tick = util::suffixed("SysTick", sender);
            let section = util::link_section("text", sender);
            items.push(quote!(
                #[no_mangle]
                #cfg_sender
                #section
                unsafe fn #sys_tick() {
                    use rtfm::Mutex as _;

                    /// The priority of this handler
                    const PRIORITY: u8 = #priority;

                    rtfm::export::run(PRIORITY, || {
                        while let Some((task, index)) = (#tq {
                            // NOTE dynamic priority is always the static priority at this point
                            priority: &rtfm::export::Priority::new(PRIORITY),
                        })
                        // NOTE `inline(always)` produces faster and smaller code
                            .lock(#[inline(always)]
                                  |tq| tq.dequeue())
                        {
                            match task {
                                #(#arms)*
                            }
                        }
                    });
                }
            ));
        }
    }

    items
}
