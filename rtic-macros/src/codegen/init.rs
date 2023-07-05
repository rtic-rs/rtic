use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    analyze::Analysis,
    codegen::{local_resources_struct, module},
    syntax::{ast::App, Context},
};

/// Generates support code for `#[init]` functions
pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    let init = &app.init;
    let name = &init.name;

    let mut root_init = vec![];

    let context = &init.context;
    let attrs = &init.attrs;
    let stmts = &init.stmts;
    let shared = &init.user_shared_struct;
    let shared_vis = &app.shared_resources_vis;
    let local = &init.user_local_struct;
    let local_vis = &app.local_resources_vis;

    let shared_resources: Vec<_> = app
        .shared_resources
        .iter()
        .map(|(k, v)| {
            let ty = &v.ty;
            let cfgs = &v.cfgs;
            let docs = &v.docs;
            let vis = &v.vis;
            quote!(
                #(#cfgs)*
                #(#docs)*
                #vis #k: #ty,
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
            let vis = &v.vis;
            quote!(
                #(#cfgs)*
                #(#docs)*
                #vis #k: #ty,
            )
        })
        .collect();

    root_init.push(quote! {
        #[doc = r"Shared resources"]
        #shared_vis struct #shared {
            #(#shared_resources)*
        }

        #[doc = r"Local resources"]
        #local_vis struct #local {
            #(#local_resources)*
        }
    });

    // let locals_pat = locals_pat.iter();

    let user_init_return = quote! {#shared, #local};

    let user_init = if !init.is_extern {
        Some(quote!(
            #(#attrs)*
            #[inline(always)]
            #[allow(non_snake_case)]
            fn #name(#context: #name::Context) -> (#user_init_return) {
                #(#stmts)*
            }
        ))
    } else {
        None
    };

    let mut mod_app = None;

    // `${task}Locals`
    if !init.args.local_resources.is_empty() {
        let (item, constructor) = local_resources_struct::codegen(Context::Init, app);

        root_init.push(item);

        mod_app = Some(constructor);
    }

    root_init.push(module::codegen(Context::Init, app, analysis));

    quote!(
        #mod_app

        #(#root_init)*

        #user_init
    )
}
