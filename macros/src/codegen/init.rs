use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{locals, module, resources_struct, util},
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
    Option<TokenStream2>,
    // call_init -- the call to the user `#[init]` if there's one
    Option<TokenStream2>,
);

/// Generates support code for `#[init]` functions
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> CodegenResult {
    if !app.inits.is_empty() {
        let init = &app.inits.first().unwrap();
        let mut needs_lt = false;
        let name = &init.name;

        let mut root_init = vec![];

        let late_fields = analysis
            .late_resources
            .iter()
            .flat_map(|resources| {
                resources.iter().map(|name| {
                    let ty = &app.late_resources[name].ty;
                    let cfgs = &app.late_resources[name].cfgs;

                    quote!(
                        #(#cfgs)*
                        pub #name: #ty
                    )
                })
            })
            .collect::<Vec<_>>();

        let late_resources = util::late_resources_ident(&name);

        root_init.push(quote!(
            /// Resources initialized at runtime
            #[allow(non_snake_case)]
            pub struct #late_resources {
                #(#late_fields),*
            }
        ));

        let monotonic_types: Vec<_> = app
            .monotonics
            .iter()
            .map(|(_, monotonic)| {
                let mono = &monotonic.ty;
                quote! {#mono}
            })
            .collect();
        let monotonics = util::monotonics_ident(&name);

        root_init.push(quote!(
            /// Monotonics used by the system
            #[allow(non_snake_case)]
            pub struct #monotonics(
                #(#monotonic_types),*
            );
        ));

        let mut locals_pat = None;
        let mut locals_new = None;
        if !init.locals.is_empty() {
            let (struct_, pat) = locals::codegen(Context::Init, &init.locals, app);

            locals_new = Some(quote!(#name::Locals::new()));
            locals_pat = Some(pat);
            root_init.push(struct_);
        }

        let context = &init.context;
        let attrs = &init.attrs;
        let stmts = &init.stmts;
        let locals_pat = locals_pat.iter();

        let mut user_init_return = vec![quote! {#name::LateResources}];
        if !app.monotonics.is_empty() {
            user_init_return.push(quote! {#name::Monotonics});
        }

        let user_init = Some(quote!(
            #(#attrs)*
            #[allow(non_snake_case)]
            fn #name(#(#locals_pat,)* #context: #name::Context) -> (#(#user_init_return,)*) {
                #(#stmts)*
            }
        ));

        let mut mod_app = None;
        if !init.args.resources.is_empty() {
            let (item, constructor) = resources_struct::codegen(Context::Init, &mut needs_lt, app);

            root_init.push(item);
            mod_app = Some(constructor);
        }

        let app_name = &app.name;
        let app_path = quote! {crate::#app_name};
        let locals_new = locals_new.iter();
        let call_init = Some(
            quote!(let (late, monotonics) = #app_path::#name(#(#locals_new,)* #name::Context::new(core.into()));),
        );

        root_init.push(module::codegen(
            Context::Init,
            needs_lt,
            app,
            analysis,
            extra,
        ));

        (mod_app, root_init, user_init, call_init)
    } else {
        (None, vec![], None, None)
    }
}
