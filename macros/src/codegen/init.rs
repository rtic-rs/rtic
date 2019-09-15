use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{locals, module, resources_struct, util},
};

/// Generates support code for `#[init]` functions
pub fn codegen(
    core: u8,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // const_app_idle -- the `${init}Resources` constructor
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
) {
    if let Some(init) = app.inits.get(&core) {
        let cfg_core = util::cfg_core(core, app.args.cores);
        let mut needs_lt = false;
        let name = &init.name;

        let mut root_init = vec![];

        let ret = {
            let late_fields = analysis
                .late_resources
                .get(&core)
                .map(|resources| {
                    resources
                        .iter()
                        .map(|name| {
                            let ty = &app.late_resources[name].ty;

                            quote!(pub #name: #ty)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or(vec![]);

            if !late_fields.is_empty() {
                let late_resources = util::late_resources_ident(&name);

                root_init.push(quote!(
                    /// Resources initialized at runtime
                    #cfg_core
                    #[allow(non_snake_case)]
                    pub struct #late_resources {
                        #(#late_fields),*
                    }
                ));

                Some(quote!(-> #name::LateResources))
            } else {
                None
            }
        };

        let mut locals_pat = None;
        let mut locals_new = None;
        if !init.locals.is_empty() {
            let (struct_, pat) = locals::codegen(Context::Init(core), &init.locals, core, app);

            locals_new = Some(quote!(#name::Locals::new()));
            locals_pat = Some(pat);
            root_init.push(struct_);
        }

        let context = &init.context;
        let attrs = &init.attrs;
        let stmts = &init.stmts;
        let section = util::link_section("text", core);
        let locals_pat = locals_pat.iter();
        let user_init = Some(quote!(
            #(#attrs)*
            #cfg_core
            #[allow(non_snake_case)]
            #section
            fn #name(#(#locals_pat,)* #context: #name::Context) #ret {
                #(#stmts)*
            }
        ));

        let mut const_app = None;
        if !init.args.resources.is_empty() {
            let (item, constructor) =
                resources_struct::codegen(Context::Init(core), 0, &mut needs_lt, app, analysis);

            root_init.push(item);
            const_app = Some(constructor);
        }

        let locals_new = locals_new.iter();
        let call_init =
            Some(quote!(let late = #name(#(#locals_new,)* #name::Context::new(core.into()));));

        root_init.push(module::codegen(Context::Init(core), needs_lt, app, extra));

        (const_app, root_init, user_init, call_init)
    } else {
        (None, vec![], None, None)
    }
}
