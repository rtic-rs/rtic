use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{local_resources_struct, module},
};

type CodegenResult = (
    // mod_app_idle -- the `${init}Resources` constructor
    Option<TokenStream2>,
    // root_init -- items that must be placed in the root of the crate:
    // - the `${init}Locals` struct
    // - the `${init}Resources` struct
    // - the `${init}LateResources` struct
    // - the `${init}` module, which contains types like `${init}::Context`
    Vec<TokenStream2>,
    // user_init -- the `#[init]` function written by the user
    TokenStream2,
    // call_init -- the call to the user `#[init]`
    TokenStream2,
);

/// Generates support code for `#[init]` functions
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> CodegenResult {
    let init = &app.init;
    let mut local_needs_lt = false;
    let name = &init.name;

    let mut root_init = vec![];

    let context = &init.context;
    let attrs = &init.attrs;
    let stmts = &init.stmts;
    let shared = &init.user_shared_struct;
    let local = &init.user_local_struct;

    let shared_resources: Vec<_> = app
        .shared_resources
        .iter()
        .map(|(k, v)| {
            let ty = &v.ty;
            let cfgs = &v.cfgs;
            let docs = &v.docs;
            quote!(
                #(#cfgs)*
                #(#docs)*
                #k: #ty,
            )
        })
        .collect();
    let local_resources: Vec<_> = app
        .local_resources
        .iter()
        .map(|(k, v)| {
            let ty = &v.ty;
            let cfgs = &v.cfgs;
            let docs = &v.docs;
            quote!(
                #(#cfgs)*
                #(#docs)*
                #k: #ty,
            )
        })
        .collect();
    root_init.push(quote! {
        struct #shared {
            #(#shared_resources)*
        }

        struct #local {
            #(#local_resources)*
        }
    });

    // let locals_pat = locals_pat.iter();

    let user_init_return = quote! {#shared, #local, #name::Monotonics};

    let user_init = quote!(
        #(#attrs)*
        #[inline(always)]
        #[allow(non_snake_case)]
        fn #name(#context: #name::Context) -> (#user_init_return) {
            #(#stmts)*
        }
    );

    let mut mod_app = None;

    // `${task}Locals`
    if !init.args.local_resources.is_empty() {
        let (item, constructor) =
            local_resources_struct::codegen(Context::Init, &mut local_needs_lt, app);

        root_init.push(item);

        mod_app = Some(constructor);
    }

    // let locals_new = locals_new.iter();
    let call_init = quote! {
        let (shared_resources, local_resources, mut monotonics) = #name(#name::Context::new(core.into()));
    };

    root_init.push(module::codegen(
        Context::Init,
        false,
        local_needs_lt,
        app,
        analysis,
        extra,
    ));

    (mod_app, root_init, user_init, call_init)
}
