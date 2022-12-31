use crate::syntax::ast::App;
use crate::{analyze::Analysis, codegen::util};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut items = vec![];

    let interrupts = &analysis.interrupts_async;

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
        if channel
            .tasks
            .iter()
            .map(|task_name| !app.software_tasks[task_name].is_async)
            .all(|is_not_async| is_not_async)
        {
            // check if all tasks are not async, if so don't generate this.
            continue;
        }

        let mut stmts = vec![];
        let device = &app.args.device;
        let enum_ = util::interrupt_ident();
        let interrupt = util::suffixed(&interrupts[&level].0.to_string());

        for name in channel
            .tasks
            .iter()
            .filter(|name| app.software_tasks[*name].is_async)
        {
            let exec_name = util::internal_task_ident(name, "EXEC");
            let prio_name = util::internal_task_ident(name, "PRIORITY");
            let task = &app.software_tasks[name];
            // let cfgs = &task.cfgs;
            let (_, tupled, pats, input_types) = util::regroup_inputs(&task.inputs);
            let executor_run_ident = util::executor_run_ident(name);

            let n = util::capacity_literal(channel.capacity as usize + 1);
            let rq = util::rq_async_ident(name);
            let (rq_ty, rq_expr) = {
                (
                    quote!(rtic::export::ASYNCRQ<#input_types, #n>),
                    quote!(rtic::export::Queue::new()),
                )
            };

            items.push(quote!(
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #[allow(non_upper_case_globals)]
                static #rq: rtic::RacyCell<#rq_ty> = rtic::RacyCell::new(#rq_expr);
            ));

            stmts.push(quote!(
                if !(&*#exec_name.get()).is_running() {
                     if let Some(#tupled) = rtic::export::interrupt::free(|_| (&mut *#rq.get_mut()).dequeue()) {

                        // The async executor needs a static priority
                        #prio_name.get_mut().write(rtic::export::Priority::new(PRIORITY));
                        let priority: &'static _ = &*#prio_name.get();

                        (&mut *#exec_name.get_mut()).spawn(#name(#name::Context::new(priority) #(,#pats)*));
                        #executor_run_ident.store(true, core::sync::atomic::Ordering::Relaxed);
                    }
                }

                if #executor_run_ident.load(core::sync::atomic::Ordering::Relaxed) {
                    #executor_run_ident.store(false, core::sync::atomic::Ordering::Relaxed);
                    if (&mut *#exec_name.get_mut()).poll(||  {
                        #executor_run_ident.store(true, core::sync::atomic::Ordering::Release);
                        rtic::pend(#device::#enum_::#interrupt);
                    }) && !rtic::export::interrupt::free(|_| (&*#rq.get_mut()).is_empty()) {
                        // If the ready queue is not empty and the executor finished, restart this
                        // dispatch to check if the executor should be restarted.
                        rtic::pend(#device::#enum_::#interrupt);
                    }
                }
            ));
        }

        let doc = format!(
            "Interrupt handler to dispatch async tasks at priority {}",
            level
        );
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
