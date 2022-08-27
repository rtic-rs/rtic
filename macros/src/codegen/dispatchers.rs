use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let interrupts = &analysis.interrupts;

    // Generate executor definition and priority in global scope
    for (name, task) in app.software_tasks.iter() {
        if task.is_async {
            let type_name = util::internal_task_ident(name, "F");
            let exec_name = util::internal_task_ident(name, "EXEC");
            let prio_name = util::internal_task_ident(name, "PRIORITY");

            items.push(quote!(
                #[allow(non_camel_case_types)]
                type #type_name = impl core::future::Future + 'static;
                #[allow(non_upper_case_globals)]
                static #exec_name:
                    rtic::RacyCell<rtic::export::executor::AsyncTaskExecutor<#type_name>> =
                        rtic::RacyCell::new(rtic::export::executor::AsyncTaskExecutor::new());

                // The executors priority, this can be any value - we will overwrite it when we
                // start a task
                #[allow(non_upper_case_globals)]
                static #prio_name: rtic::RacyCell<rtic::export::Priority> =
                        unsafe { rtic::RacyCell::new(rtic::export::Priority::new(0)) };
            ));
        }
    }

    for (&level, channel) in &analysis.channels {
        let mut stmts = vec![];

        let variants = channel
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

        // For future use
        // let doc = format!(
        //     "Software tasks to be dispatched at priority level {}",
        //     level,
        // );
        let t = util::spawn_t_ident(level);
        items.push(quote!(
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            // #[doc = #doc]
            #[doc(hidden)]
            pub enum #t {
                #(#variants,)*
            }
        ));

        let n = util::capacity_literal(channel.capacity as usize + 1);
        let rq = util::rq_ident(level);
        let (rq_ty, rq_expr) = {
            (
                quote!(rtic::export::SCRQ<#t, #n>),
                quote!(rtic::export::Queue::new()),
            )
        };

        // For future use
        // let doc = format!(
        //     "Queue of tasks ready to be dispatched at priority level {}",
        //     level
        // );
        items.push(quote!(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            static #rq: rtic::RacyCell<#rq_ty> = rtic::RacyCell::new(#rq_expr);
        ));

        let device = &extra.device;
        let enum_ = util::interrupt_ident();
        let interrupt = util::suffixed(&interrupts[&level].0.to_string());
        let arms = channel
            .tasks
            .iter()
            .map(|name| {
                let task = &app.software_tasks[name];
                let cfgs = &task.cfgs;
                let fq = util::fq_ident(name);
                let inputs = util::inputs_ident(name);
                let (_, tupled, pats, _) = util::regroup_inputs(&task.inputs);
                let exec_name = util::internal_task_ident(name, "EXEC");
                let prio_name = util::internal_task_ident(name, "PRIORITY");

                if task.is_async {
                    let executor_run_ident = util::executor_run_ident(name);

                    quote!(
                        #(#cfgs)*
                        #t::#name => {
                            if !(&mut *#exec_name.get_mut()).is_running() {
                                let #tupled =
                                    (&*#inputs
                                    .get())
                                    .get_unchecked(usize::from(index))
                                    .as_ptr()
                                    .read();
                                (&mut *#fq.get_mut()).split().0.enqueue_unchecked(index);

                                // The async executor needs a static priority
                                #prio_name.get_mut().write(rtic::export::Priority::new(PRIORITY));
                                let priority: &'static _ = &*#prio_name.get();

                                (&mut *#exec_name.get_mut()).spawn(#name(#name::Context::new(priority) #(,#pats)*));
                                #executor_run_ident.store(true, core::sync::atomic::Ordering::Relaxed);
                            } else {
                                retry_queue.push_unchecked((#t::#name, index));
                            }
                        }
                    )
                } else {
                    quote!(
                        #(#cfgs)*
                        #t::#name => {
                            let #tupled =
                                (&*#inputs
                                .get())
                                .get_unchecked(usize::from(index))
                                .as_ptr()
                                .read();
                            (&mut *#fq.get_mut()).split().0.enqueue_unchecked(index);
                            let priority = &rtic::export::Priority::new(PRIORITY);
                            #name(
                                #name::Context::new(priority)
                                #(,#pats)*
                            )
                        }
                    )
                }
            })
            .collect::<Vec<_>>();

        let n_executors = channel
            .tasks
            .iter()
            .map(|name| {
                let task = &app.software_tasks[name];
                if task.is_async {
                    1
                } else {
                    0
                }
            })
            .sum::<usize>()
            .max(1);

        // TODO: This `retry_queue` comes from the current design of the dispatcher queue handling.
        // To remove this we would need to redesign how the dispatcher handles queues, and this can
        // be done as an optimization later.
        //
        // The core issue is that we should only dequeue the ready queue if the exexutor associated
        // to the task is not running. As it is today this queue is blindly dequeued, see the
        // `while let Some(...) = (&mut *#rq.get_mut())...` a few lines down. The current "hack" is
        // to just requeue the executor run if it should not have been dequeued. This needs however
        // to be done after the ready queue has been exhausted.
        stmts.push(quote!(
            let mut retry_queue: rtic::export::Vec<_, #n_executors> = rtic::export::Vec::new();
        ));

        stmts.push(quote!(
            while let Some((task, index)) = (&mut *#rq.get_mut()).split().1.dequeue() {
                match task {
                    #(#arms)*
                }
            }
        ));

        for name in channel
            .tasks
            .iter()
            .filter(|name| app.software_tasks[*name].is_async)
        {
            let exec_name = util::internal_task_ident(name, "EXEC");

            let executor_run_ident = util::executor_run_ident(name);
            stmts.push(quote!(
                if #executor_run_ident.load(core::sync::atomic::Ordering::Relaxed) {
                    #executor_run_ident.store(false, core::sync::atomic::Ordering::Relaxed);
                    if (&mut *#exec_name.get_mut()).poll(||  {
                        #executor_run_ident.store(true, core::sync::atomic::Ordering::Release);
                        rtic::pend(#device::#enum_::#interrupt);
                    }) && !retry_queue.is_empty() {
                        // If the retry queue is not empty and the executor finished, restart this
                        // dispatch to check if the executor should be restarted.
                        rtic::pend(#device::#enum_::#interrupt);
                    }
                }
            ));
        }

        stmts.push(quote!(
            while let Some((task, index)) = retry_queue.pop() {
                rtic::export::interrupt::free(|_| {
                    (&mut *#rq.get_mut()).enqueue_unchecked((task, index));
                });
            }
        ));

        let doc = format!("Interrupt handler to dispatch tasks at priority {}", level);
        let attribute = &interrupts[&level].1.attrs;
        items.push(quote!(
            #[allow(non_snake_case)]
            #[doc = #doc]
            #[no_mangle]
            #(#attribute)*
            unsafe fn #interrupt() {
                /// The priority of this interrupt handler
                const PRIORITY: u8 = #level;

                rtic::export::run(PRIORITY, || {
                    #(#stmts)*
                });
            }
        ));
    }

    items
}
