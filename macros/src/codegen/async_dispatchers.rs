use crate::syntax::ast::App;
use crate::{analyze::Analysis, codegen::util};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    let mut items = vec![];

    let interrupts = &analysis.interrupts;

    // Generate executor definition and priority in global scope
    for (name, _) in app.software_tasks.iter() {
        let type_name = util::internal_task_ident(name, "F");
        let exec_name = util::internal_task_ident(name, "EXEC");

        items.push(quote!(
            #[allow(non_camel_case_types)]
            type #type_name = impl core::future::Future;
            #[allow(non_upper_case_globals)]
            static #exec_name: rtic::export::executor::AsyncTaskExecutor<#type_name> =
                rtic::export::executor::AsyncTaskExecutor::new();
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
            let device = &app.args.device;
            let enum_ = util::interrupt_ident();

            quote!(rtic::pend(#device::#enum_::#dispatcher_name);)
        } else {
            // For 0 priority tasks we don't need to pend anything
            quote!()
        };

        for name in channel.tasks.iter() {
            let exec_name = util::internal_task_ident(name, "EXEC");
            // let task = &app.software_tasks[name];
            // let cfgs = &task.cfgs;

            stmts.push(quote!(
                if #exec_name.check_and_clear_pending() {
                    #exec_name.poll(|| {
                        #exec_name.set_pending();
                        #pend_interrupt
                    });
                }
            ));
        }

        if level > 0 {
            let doc = format!("Interrupt handler to dispatch async tasks at priority {level}");
            let attribute = &interrupts.get(&level).expect("UNREACHABLE").1.attrs;
            items.push(quote!(
                #[allow(non_snake_case)]
                #[doc = #doc]
                #[no_mangle]
                #(#attribute)*
                unsafe fn #dispatcher_name() {
                    /// The priority of this interrupt handler
                    const PRIORITY: u8 = #level;

                    rtic::export::run(PRIORITY, || {
                        #(#stmts)*
                    });
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
