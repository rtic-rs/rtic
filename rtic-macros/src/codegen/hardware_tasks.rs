use crate::syntax::{ast::App, Context};
use crate::{
    analyze::Analysis,
    codegen::{
        bindings::{interrupt_entry, interrupt_exit, async_config},
        local_resources_struct, module, shared_resources_struct,
    },
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generate support code for hardware tasks (`#[exception]`s and `#[interrupt]`s)
pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    let mut mod_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];

    for (name, task) in &app.hardware_tasks {
        let symbol = task.args.binds.clone();
        let priority = task.args.priority;
        let cfgs = &task.cfgs;
        let attrs = &task.attrs;
        let entry_stmts = interrupt_entry(app, analysis);
        let exit_stmts = interrupt_exit(app, analysis);
        let config = handler_config(app, analysis, symbol.clone());

        mod_app.push(quote!(
            #[allow(non_snake_case)]
            #[no_mangle]
            #(#attrs)*
            #(#cfgs)*
            #(#config)*
            unsafe fn #symbol() {
                #(#entry_stmts)*

                const PRIORITY: u8 = #priority;

                rtic::export::run(PRIORITY, || {
                    #name(
                        #name::Context::new()
                    )
                });

                #(#exit_stmts)*
            }
        ));

        // `${task}Locals`
        if !task.args.local_resources.is_empty() {
            let (item, constructor) =
                local_resources_struct::codegen(Context::HardwareTask(name), app);

            root.push(item);

            mod_app.push(constructor);
        }

        // `${task}Resources`
        if !task.args.shared_resources.is_empty() {
            let (item, constructor) =
                shared_resources_struct::codegen(Context::HardwareTask(name), app);

            root.push(item);

            mod_app.push(constructor);
        }

        // Module generation...

        root.push(module::codegen(Context::HardwareTask(name), app, analysis));

        // End module generation

        if !task.is_extern {
            let attrs = &task.attrs;
            let context = &task.context;
            let stmts = &task.stmts;
            user_tasks.push(quote!(
                #(#attrs)*
                #[allow(non_snake_case)]
                fn #name(#context: #name::Context) {
                    use rtic::Mutex as _;
                    use rtic::mutex::prelude::*;

                    #(#stmts)*
                }
            ));
        }
    }

    quote!(
        #(#mod_app)*

        #(#root)*

        #(#user_tasks)*
    )
}
