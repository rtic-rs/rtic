use crate::syntax::{analyze::Ownership, ast::App};
use crate::{analyze::Analysis, codegen::util};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::bindings::impl_mutex;

/// Generates `static` variables and shared resource proxies
pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    let mut mod_app = vec![];
    let mut mod_resources = vec![];

    for (name, res) in &app.shared_resources {
        let cfgs = &res.cfgs;
        let ty = &res.ty;
        let mangled_name = &util::static_shared_resource_ident(name);

        let attrs = &res.attrs;

        // late resources in `util::link_section_uninit`
        // unless user specifies custom link section
        let section = if attrs.iter().any(|attr| {
            let is_link_section = attr.path().is_ident("link_section");
            let is_unsafe = attr.path().is_ident("unsafe");
            let is_embedded_link_section = match attr.parse_args() {
                Ok(syn::Expr::Assign(assign)) => match &*assign.left {
                    syn::Expr::Path(path) => path.path.is_ident("link_section"),
                    _ => false,
                },
                _ => false,
            };

            is_link_section || (is_unsafe && is_embedded_link_section)
        }) {
            None
        } else {
            Some(util::link_section_uninit())
        };

        // For future use
        // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
        mod_app.push(quote!(
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            // #[doc = #doc]
            #[doc(hidden)]
            #(#attrs)*
            #(#cfgs)*
            #section
            static #mangled_name: rtic::RacyCell<core::mem::MaybeUninit<#ty>> = rtic::RacyCell::new(core::mem::MaybeUninit::uninit());
        ));

        // For future use
        // let doc = format!(" RTIC internal: {}:{}", file!(), line!());

        let shared_name = util::need_to_lock_ident(name);

        if !res.properties.lock_free {
            mod_resources.push(quote!(
                // #[doc = #doc]
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #(#cfgs)*
                pub struct #shared_name<'a> {
                    __rtic_internal_p: ::core::marker::PhantomData<&'a ()>,
                }

                #(#cfgs)*
                impl<'a> #shared_name<'a> {
                    #[inline(always)]
                    pub unsafe fn new() -> Self {
                        #shared_name { __rtic_internal_p: ::core::marker::PhantomData }
                    }
                }
            ));

            let ptr = quote!(
                #(#cfgs)*
                #mangled_name.get_mut() as *mut _
            );

            let ceiling = match analysis.ownerships.get(name) {
                Some(Ownership::Owned { priority } | Ownership::CoOwned { priority }) => *priority,
                Some(Ownership::Contended { ceiling }) => *ceiling,
                None => 0,
            };

            // For future use
            // let doc = format!(" RTIC internal ({} resource): {}:{}", doc, file!(), line!());

            mod_app.push(impl_mutex(
                app,
                analysis,
                cfgs,
                true,
                &shared_name,
                &quote!(#ty),
                ceiling,
                &ptr,
            ));
        }
    }

    let mod_resources = if mod_resources.is_empty() {
        quote!()
    } else {
        quote!(mod shared_resources {
            #(#mod_resources)*
        })
    };

    quote!(
        #(#mod_app)*

        #mod_resources
    )
}
