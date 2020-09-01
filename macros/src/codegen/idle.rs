use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{locals, module, resources_struct},
};

/// Generates support code for `#[idle]` functions
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // const_app_idle -- the `${idle}Resources` constructor
    Option<TokenStream2>,
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
    if app.idles.len() > 0 {
        let idle = &app.idles.first().unwrap();
        let mut needs_lt = false;
        let mut const_app = None;
        let mut root_idle = vec![];
        let mut locals_pat = None;
        let mut locals_new = None;

        if !idle.args.resources.is_empty() {
            let (item, constructor) =
                resources_struct::codegen(Context::Idle, 0, &mut needs_lt, app, analysis);

            root_idle.push(item);
            const_app = Some(constructor);
        }

        let name = &idle.name;
        if !idle.locals.is_empty() {
            let (locals, pat) = locals::codegen(Context::Idle, &idle.locals, app);

            locals_new = Some(quote!(#name::Locals::new()));
            locals_pat = Some(pat);
            root_idle.push(locals);
        }

        root_idle.push(module::codegen(Context::Idle, needs_lt, app, extra));

        let attrs = &idle.attrs;
        let context = &idle.context;
        let stmts = &idle.stmts;
        let locals_pat = locals_pat.iter();
        let user_idle = Some(quote!(
            #(#attrs)*
            #[allow(non_snake_case)]
            fn #name(#(#locals_pat,)* #context: #name::Context) -> ! {
                use rtic::Mutex as _;

                #(#stmts)*
            }
        ));

        let locals_new = locals_new.iter();
        let call_idle = quote!(crate::#name(
            #(#locals_new,)*
            #name::Context::new(&rtic::export::Priority::new(0))
        ));

        (const_app, root_idle, user_idle, call_idle)
    } else {
        (
            None,
            vec![],
            None,
            quote!(loop {
                rtic::export::wfi()
            }),
        )
    }
}
