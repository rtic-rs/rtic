use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{local_resources_struct, module, shared_resources_struct},
};

/// Generates support code for `#[idle]` functions
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
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
        let mut shared_needs_lt = false;
        let mut local_needs_lt = false;
        let mut mod_app = vec![];
        let mut root_idle = vec![];

        let name = &idle.name;

        if !idle.args.shared_resources.is_empty() {
            let (item, constructor) =
                shared_resources_struct::codegen(Context::Idle, &mut shared_needs_lt, app);

            root_idle.push(item);
            mod_app.push(constructor);
        }

        if !idle.args.local_resources.is_empty() {
            let (item, constructor) =
                local_resources_struct::codegen(Context::Idle, &mut local_needs_lt, app);

            root_idle.push(item);

            mod_app.push(constructor);
        }

        root_idle.push(module::codegen(
            Context::Idle,
            shared_needs_lt,
            local_needs_lt,
            app,
            analysis,
            extra,
        ));

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
            #name::Context::new(&rtic::export::Priority::new(0))
        ));

        (mod_app, root_idle, user_idle, call_idle)
    } else {
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
