use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{analyze::Ownership, ast::App};

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
    // mod_resources_imports -- the `resources` module imports
    Vec<TokenStream2>,
) {
    let mut const_app = vec![];
    let mut mod_resources = vec![];
    let mut mod_resources_imports = vec![];

    for (name, res, expr, _) in app.resources(analysis) {
        let cfgs = &res.cfgs;
        let ty = &res.ty;

        {
            let section = if expr.is_none() {
                util::link_section_uninit(true)
            } else {
                None
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
                #[allow(non_upper_case_globals)]
                #(#attrs)*
                #(#cfgs)*
                #section
                static mut #name: #ty = #expr;
            ));
        }

        if let Some(Ownership::Contended { ceiling }) = analysis.ownerships.get(name) {
            mod_resources.push(quote!(
                #[allow(non_camel_case_types)]
                #(#cfgs)*
                pub struct #name<'a> {
                    priority: &'a Priority,
                }

                #(#cfgs)*
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
                quote!(
                    #(#cfgs)*
                    #name.as_mut_ptr()
                )
            } else {
                quote!(
                    #(#cfgs)*
                    &mut #name
                )
            };

            mod_resources_imports.push(quote!(
                #[allow(non_camel_case_types)]
                #(#cfgs)*
                #cfg_core
                use super::resources::#name;
            ));

            const_app.push(util::impl_mutex(
                extra,
                cfgs,
                true,
                name,
                quote!(#ty),
                *ceiling,
                ptr,
            ));
        }
    }

    let mod_resources = if mod_resources.is_empty() {
        quote!()
    } else {
        // Also import the resource module
        mod_resources_imports.push(quote!(
            use super::resources;
        ));

        quote!(mod resources {
            use rtic::export::Priority;

            #(#mod_resources)*
        })
    };

    (const_app, mod_resources, mod_resources_imports)
}
