use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
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
    // mod_app_idle -- the `${idle}Resources` constructor
    Option<TokenStream2>,
    // root_idle -- items that must be placed in the root of the crate:
    // - the `${idle}Locals` struct
    // - the `${idle}Resources` struct
    // - the `${idle}` module, which contains types like `${idle}::Context`
    Vec<TokenStream2>,
    // user_idle
    Option<TokenStream2>,
    // user_idle_imports
    Vec<TokenStream2>,
    // call_idle
    TokenStream2,
) {
    if !app.idles.is_empty() {
        let idle = &app.idles.first().unwrap();
        let mut needs_lt = false;
        let mut mod_app = None;
        let mut root_idle = vec![];
        let mut locals_pat = None;
        let mut locals_new = None;

        let mut user_idle_imports = vec![];

        let name = &idle.name;

        if !idle.args.resources.is_empty() {
            let (item, constructor) =
                resources_struct::codegen(Context::Idle, 0, &mut needs_lt, app, analysis);

            root_idle.push(item);
            mod_app = Some(constructor);

            let name_resource = format_ident!("{}Resources", name);
            user_idle_imports.push(quote!(
                    #[allow(non_snake_case)]
                    use super::#name_resource;
            ));
        }

        if !idle.locals.is_empty() {
            let (locals, pat) = locals::codegen(Context::Idle, &idle.locals, app);

            locals_new = Some(quote!(#name::Locals::new()));
            locals_pat = Some(pat);
            root_idle.push(locals);
        }

        root_idle.push(module::codegen(
            Context::Idle,
            needs_lt,
            app,
            analysis,
            extra,
        ));

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
        user_idle_imports.push(quote!(
            #(#attrs)*
            #[allow(non_snake_case)]
            use super::#name;
        ));

        let locals_new = locals_new.iter();
        let call_idle = quote!(crate::#name(
            #(#locals_new,)*
            #name::Context::new(&rtic::export::Priority::new(0))
        ));

        (mod_app, root_idle, user_idle, user_idle_imports, call_idle)
    } else {
        (
            None,
            vec![],
            None,
            vec![],
            quote!(loop {
                rtic::export::wfi()
            }),
        )
    }
}
