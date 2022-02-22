use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{local_resources_struct, module, shared_resources_struct},
};

/// Generate support code for hardware tasks (`#[exception]`s and `#[interrupt]`s)
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // mod_app_hardware_tasks -- interrupt handlers and `${task}Resources` constructors
    Vec<TokenStream2>,
    // root_hardware_tasks -- items that must be placed in the root of the crate:
    // - `${task}Locals` structs
    // - `${task}Resources` structs
    // - `${task}` modules
    Vec<TokenStream2>,
    // user_hardware_tasks -- the `#[task]` functions written by the user
    Vec<TokenStream2>,
) {
    let mut mod_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];

    for (name, task) in &app.hardware_tasks {
        let symbol = task.args.binds.clone();
        let priority = task.args.priority;
        let cfgs = &task.cfgs;
        let attrs = &task.attrs;

        mod_app.push(quote!(
            #[allow(non_snake_case)]
            #[no_mangle]
            #(#attrs)*
            #(#cfgs)*
            unsafe fn #symbol() {
                const PRIORITY: u8 = #priority;

                rtic::export::run(PRIORITY, || {
                    #name(
                        #name::Context::new(&rtic::export::Priority::new(PRIORITY))
                    )
                });
            }
        ));

        let mut shared_needs_lt = false;
        let mut local_needs_lt = false;

        // `${task}Locals`
        if !task.args.local_resources.is_empty() {
            let (item, constructor) = local_resources_struct::codegen(
                Context::HardwareTask(name),
                &mut local_needs_lt,
                app,
            );

            root.push(item);

            mod_app.push(constructor);
        }

        // `${task}Resources`
        if !task.args.shared_resources.is_empty() {
            let (item, constructor) = shared_resources_struct::codegen(
                Context::HardwareTask(name),
                &mut shared_needs_lt,
                app,
            );

            root.push(item);

            mod_app.push(constructor);
        }

        root.push(module::codegen(
            Context::HardwareTask(name),
            shared_needs_lt,
            local_needs_lt,
            app,
            analysis,
            extra,
        ));

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

    (mod_app, root, user_tasks)
}
