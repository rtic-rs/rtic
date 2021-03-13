use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{analyze::Ownership, ast::App};

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates `static` variables and resource proxies
/// Early resources are stored in `RacyCell<T>`
/// Late resource are stored in `RacyCell<MaybeUninit<T>>`
///
/// Safety:
/// - RacyCell<T> access is `unsafe`.
/// - RacyCell<MaybeUninit> is always written to before user access, thus
//    the generated code for user access can safely `assume_init`.
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // mod_app -- the `static` variables behind the proxies
    Vec<TokenStream2>,
    // mod_resources -- the `resources` module
    TokenStream2,
) {
    let mut mod_app = vec![];
    let mut mod_resources = vec![];

    for (name, res, expr, _) in app.resources(analysis) {
        let cfgs = &res.cfgs;
        let ty = &res.ty;
        let mangled_name = util::mark_internal_ident(&name);

        {
            // TODO: do we really need this in the single core case
            // late resources in `util::link_section_uninit`
            let section = if expr.is_none() {
                util::link_section_uninit(true)
            } else {
                None
            };

            // resource type and assigned value
            let (ty, expr) = if let Some(expr) = expr {
                // early resource
                (
                    quote!(rtic::RacyCell<#ty>),
                    quote!(rtic::RacyCell::new(#expr)),
                )
            } else {
                // late resource
                (
                    quote!(rtic::RacyCell<core::mem::MaybeUninit<#ty>>),
                    quote!(rtic::RacyCell::new(core::mem::MaybeUninit::uninit())),
                )
            };

            let attrs = &res.attrs;

            let doc = format!(" RTIC internal: {}:{}", file!(), line!());
            mod_app.push(quote!(
                #[allow(non_upper_case_globals)]
                #[doc = #doc]
                #(#attrs)*
                #(#cfgs)*
                #section
                static #mangled_name: #ty = #expr;
            ));
        }

        let r_prop = &res.properties;
        let doc = format!(" RTIC internal: {}:{}", file!(), line!());

        if !r_prop.task_local && !r_prop.lock_free {
            mod_resources.push(quote!(
                #[doc = #doc]
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

            let (ptr, _doc) = if expr.is_none() {
                // late resource
                (
                    quote!(
                        #(#cfgs)*
                        #mangled_name.get_mut_unchecked().as_mut_ptr()
                    ),
                    "late",
                )
            } else {
                // early resource
                (
                    quote!(
                        #(#cfgs)*
                        #mangled_name.get_mut_unchecked()
                    ),
                    "early",
                )
            };

            let ceiling = match analysis.ownerships.get(name) {
                Some(Ownership::Owned { priority }) => *priority,
                Some(Ownership::CoOwned { priority }) => *priority,
                Some(Ownership::Contended { ceiling }) => *ceiling,
                None => 0,
            };

            // let doc = format!(" RTIC internal ({} resource): {}:{}", doc, file!(), line!());

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
