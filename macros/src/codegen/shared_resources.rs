use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{analyze::Ownership, ast::App};

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates `static` variables and shared resource proxies
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

    for (name, res) in &app.shared_resources {
        let cfgs = &res.cfgs;
        let ty = &res.ty;
        let mangled_name = &util::static_shared_resource_ident(&name);

        // late resources in `util::link_section_uninit`
        let section = util::link_section_uninit();
        let attrs = &res.attrs;

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

        if !res.properties.lock_free {
            mod_resources.push(quote!(
                // #[doc = #doc]
                #[doc(hidden)]
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

            let ptr = quote!(
                #(#cfgs)*
                #mangled_name.get_mut() as *mut _
            );

            let ceiling = match analysis.ownerships.get(name) {
                Some(Ownership::Owned { priority }) => *priority,
                Some(Ownership::CoOwned { priority }) => *priority,
                Some(Ownership::Contended { ceiling }) => *ceiling,
                None => 0,
            };

            // For future use
            // let doc = format!(" RTIC internal ({} resource): {}:{}", doc, file!(), line!());

            mod_app.push(util::impl_mutex(
                extra,
                cfgs,
                true,
                &name,
                quote!(#ty),
                ceiling,
                ptr,
            ));
        }
    }

    let mod_resources = if mod_resources.is_empty() {
        quote!()
    } else {
        quote!(mod shared_resources {
            use rtic::export::Priority;

            #(#mod_resources)*
        })
    };

    (mod_app, mod_resources)
}
