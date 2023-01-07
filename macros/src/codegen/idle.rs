use crate::syntax::{ast::App, Context};
use crate::{
    analyze::Analysis,
    codegen::{local_resources_struct, module, shared_resources_struct},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates support code for `#[idle]` functions
pub fn codegen(
    app: &App,
    analysis: &Analysis,
) -> (
    // mod_app_idle -- the `${idle}Resources` constructor
    Vec<TokenStream2>,
    // root_idle -- items that must be placed in the root of the crate:
    // - the `${idle}Locals` struct
    // - the `${idle}Resources` struct
    // - the `${idle}` module, which contains types like `${idle}::Context`
    Vec<TokenStream2>,
    // user_idle
    Option<TokenStream2>,
    // call_idle
    TokenStream2,
) {
    if let Some(idle) = &app.idle {
        let mut mod_app = vec![];
        let mut root_idle = vec![];

        let name = &idle.name;

        if !idle.args.shared_resources.is_empty() {
            let (item, constructor) = shared_resources_struct::codegen(Context::Idle, app);

            root_idle.push(item);
            mod_app.push(constructor);
        }

        if !idle.args.local_resources.is_empty() {
            let (item, constructor) = local_resources_struct::codegen(Context::Idle, app);

            root_idle.push(item);

            mod_app.push(constructor);
        }

        root_idle.push(module::codegen(Context::Idle, app, analysis));

        let attrs = &idle.attrs;
        let context = &idle.context;
        let stmts = &idle.stmts;
        let user_idle = Some(quote!(
            #(#attrs)*
            #[allow(non_snake_case)]
            fn #name(#context: #name::Context) -> ! {
                use rtic::Mutex as _;
                use rtic::mutex::prelude::*;

                #(#stmts)*
            }
        ));

        let call_idle = quote!(#name(
            #name::Context::new()
        ));

        (mod_app, root_idle, user_idle, call_idle)
    } else {
        // TODO: No idle defined, check for 0-priority tasks and generate an executor if needed

        //
        (
            vec![],
            vec![],
            None,
            quote!(loop {
                rtic::export::nop()
            }),
        )
    }
}
