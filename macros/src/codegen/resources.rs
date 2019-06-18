use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::{
    analyze::{Location, Ownership},
    ast::App,
};

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates `static [mut]` variables and resource proxies
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // const_app -- the `static [mut]` variables behind the proxies
    Vec<TokenStream2>,
    // mod_resources -- the `resources` module
    TokenStream2,
) {
    let mut const_app = vec![];
    let mut mod_resources = vec![];

    for (name, res, expr, loc) in app.resources(analysis) {
        let cfgs = &res.cfgs;
        let ty = &res.ty;

        {
            let loc_attr = match loc {
                Location::Owned {
                    core,
                    cross_initialized: false,
                } => util::cfg_core(*core, app.args.cores),

                // shared `static`s and cross-initialized resources need to be in `.shared` memory
                _ => {
                    if cfg!(feature = "heterogeneous") {
                        Some(quote!(#[rtfm::export::shared]))
                    } else {
                        None
                    }
                }
            };

            let (ty, expr) = if let Some(expr) = expr {
                (quote!(#ty), quote!(#expr))
            } else {
                (
                    quote!(core::mem::MaybeUninit<#ty>),
                    quote!(core::mem::MaybeUninit::uninit()),
                )
            };

            let attrs = &res.attrs;
            const_app.push(quote!(
                #loc_attr
                #(#attrs)*
                #(#cfgs)*
                static mut #name: #ty = #expr;
            ));
        }

        // generate a resource proxy if needed
        if res.mutability.is_some() {
            if let Some(Ownership::Shared { ceiling }) = analysis.ownerships.get(name) {
                let cfg_core = util::cfg_core(loc.core().expect("UNREACHABLE"), app.args.cores);

                mod_resources.push(quote!(
                    #(#cfgs)*
                    #cfg_core
                    pub struct #name<'a> {
                        priority: &'a Priority,
                    }

                    #(#cfgs)*
                    #cfg_core
                    impl<'a> #name<'a> {
                        #[inline(always)]
                        pub unsafe fn new(priority: &'a Priority) -> Self {
                            #name { priority }
                        }

                        #[inline(always)]
                        pub unsafe fn priority(&self) -> &Priority {
                            self.priority
                        }
                    }
                ));

                let ptr = if expr.is_none() {
                    quote!(#name.as_mut_ptr())
                } else {
                    quote!(&mut #name)
                };

                const_app.push(util::impl_mutex(
                    extra,
                    cfgs,
                    cfg_core.as_ref(),
                    true,
                    name,
                    quote!(#ty),
                    *ceiling,
                    ptr,
                ));
            }
        }
    }

    let mod_resources = if mod_resources.is_empty() {
        quote!()
    } else {
        quote!(mod resources {
            use rtfm::export::Priority;

            #(#mod_resources)*
        })
    };

    (const_app, mod_resources)
}
