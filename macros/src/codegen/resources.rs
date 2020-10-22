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
    // mod_app -- the `static [mut]` variables behind the proxies
    Vec<TokenStream2>,
    // mod_resources -- the `resources` module
    TokenStream2,
) {
    let mut mod_app = vec![];
    let mut mod_resources = vec![];

    for (name, res, expr, _) in app.resources(analysis) {
        let cfgs = &res.cfgs;
        let ty = &res.ty;
        let mangled_name = util::mangle_ident(&name);

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
            mod_app.push(quote!(
                #[allow(non_upper_case_globals)]
                #(#attrs)*
                #(#cfgs)*
                #section
                static mut #mangled_name: #ty = #expr;
            ));
        }

        let r_prop = &res.properties;
        if !r_prop.task_local && !r_prop.lock_free {
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
                    #mangled_name.as_mut_ptr()
                )
            } else {
                quote!(
                    #(#cfgs)*
                    &mut #mangled_name
                )
            };

            let ceiling = match analysis.ownerships.get(name) {
                Some(Ownership::Owned { priority }) => *priority,
                Some(Ownership::CoOwned { priority }) => *priority,
                Some(Ownership::Contended { ceiling }) => *ceiling,
                None => 0,
            };

            mod_app.push(util::impl_mutex(
                extra,
                cfgs,
                true,
                name,
                quote!(#ty),
                ceiling,
                ptr,
            ));
        }
    }

    let mod_resources = if mod_resources.is_empty() {
        quote!()
    } else {
        quote!(mod resources {
            use rtic::export::Priority;

            #(#mod_resources)*
        })
    };

    (mod_app, mod_resources)
}
