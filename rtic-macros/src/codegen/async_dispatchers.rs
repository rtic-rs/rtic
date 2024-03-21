use crate::syntax::ast::App;
use crate::{
    analyze::Analysis,
    codegen::{
        bindings::{async_entry, handler_config, interrupt_entry, interrupt_exit, interrupt_mod},
        util,
    },
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    let mut items = vec![];

    let interrupts = &analysis.interrupts;

    // Generate executor definition and priority in global scope
    for (name, _) in app.software_tasks.iter() {
        let exec_name = util::internal_task_ident(name, "EXEC");

        items.push(quote!(
            #[allow(non_upper_case_globals)]
            static #exec_name: rtic::export::executor::AsyncTaskExecutorPtr =
                rtic::export::executor::AsyncTaskExecutorPtr::new();
        ));
    }

    for (&level, channel) in &analysis.channels {
        let mut stmts = vec![];

        let dispatcher_name = if level > 0 {
            util::suffixed(&interrupts.get(&level).expect("UNREACHABLE").0.to_string())
        } else {
            util::zero_prio_dispatcher_ident()
        };

        let pend_interrupt = if level > 0 {
            let int_mod = interrupt_mod(app);

            quote!(rtic::export::pend(#int_mod::#dispatcher_name);)
        } else {
            // For 0 priority tasks we don't need to pend anything
            quote!()
        };

        for name in channel.tasks.iter() {
            let exec_name = util::internal_task_ident(name, "EXEC");
            let from_ptr_n_args =
                util::from_ptr_n_args_ident(app.software_tasks[name].inputs.len());

            // TODO: Fix cfg
            // let task = &app.software_tasks[name];
            // let cfgs = &task.cfgs;

            stmts.push(quote!(
                let exec = rtic::export::executor::AsyncTaskExecutor::#from_ptr_n_args(#name, &#exec_name);
                exec.poll(|| {
                    let exec = rtic::export::executor::AsyncTaskExecutor::#from_ptr_n_args(#name, &#exec_name);
                    exec.set_pending();
                    #pend_interrupt
                });
            ));
        }

        if level > 0 {
            let doc = format!("Interrupt handler to dispatch async tasks at priority {level}");
            let attribute = &interrupts.get(&level).expect("UNREACHABLE").1.attrs;
            let entry_stmts = interrupt_entry(app, analysis);
            let exit_stmts = interrupt_exit(app, analysis);
            let async_entry_stmts = async_entry(app, analysis, dispatcher_name.clone());
            let config = handler_config(app, analysis, dispatcher_name.clone());
            items.push(quote!(
                #[allow(non_snake_case)]
                #[doc = #doc]
                #[no_mangle]
                #(#attribute)*
                #(#config)*
                unsafe fn #dispatcher_name() {
                    #(#entry_stmts)*
                    #(#async_entry_stmts)*

                    /// The priority of this interrupt handler
                    const PRIORITY: u8 = #level;

                    rtic::export::run(PRIORITY, || {
                        #(#stmts)*
                    });

                    #(#exit_stmts)*
                }
            ));
        } else {
            items.push(quote!(
                #[allow(non_snake_case)]
                unsafe fn #dispatcher_name() -> ! {
                    loop {
                        #(#stmts)*
                    }
                }
            ));
        }
    }

    quote!(#(#items)*)
}
