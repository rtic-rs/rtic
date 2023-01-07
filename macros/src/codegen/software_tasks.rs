use crate::syntax::{ast::App, Context};
use crate::{
    analyze::Analysis,
    codegen::{local_resources_struct, module, shared_resources_struct, util},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    let mut mod_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];

    // Any task
    for (name, task) in app.software_tasks.iter() {
        let executor_ident = util::executor_run_ident(name);
        mod_app.push(quote!(
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            #[doc(hidden)]
            static #executor_ident: core::sync::atomic::AtomicBool =
                core::sync::atomic::AtomicBool::new(false);
        ));

        // `${task}Resources`

        // `${task}Locals`
        if !task.args.local_resources.is_empty() {
            let (item, constructor) =
                local_resources_struct::codegen(Context::SoftwareTask(name), app);

            root.push(item);

            mod_app.push(constructor);
        }

        if !task.args.shared_resources.is_empty() {
            let (item, constructor) =
                shared_resources_struct::codegen(Context::SoftwareTask(name), app);

            root.push(item);

            mod_app.push(constructor);
        }

        if !&task.is_extern {
            let context = &task.context;
            let attrs = &task.attrs;
            let cfgs = &task.cfgs;
            let stmts = &task.stmts;

            user_tasks.push(quote!(
                #(#attrs)*
                #(#cfgs)*
                #[allow(non_snake_case)]
                async fn #name(#context: #name::Context<'static>) {
                    use rtic::Mutex as _;
                    use rtic::mutex::prelude::*;

                    #(#stmts)*
                }
            ));
        }

        root.push(module::codegen(Context::SoftwareTask(name), app, analysis));
    }

    quote!(
        #(#mod_app)*

        #(#root)*

        #(#user_tasks)*
    )
}
