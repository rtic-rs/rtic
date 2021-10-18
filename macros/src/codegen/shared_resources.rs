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
        let mangled_name = &util::static_shared_resource_ident(name);

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

        let shared_name = util::need_to_lock_ident(name);

        if !res.properties.lock_free {
            mod_resources.push(quote!(
                // #[doc = #doc]
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #(#cfgs)*
                pub struct #shared_name<'a> {
                    priority: &'a Priority,
                }

                #(#cfgs)*
                impl<'a> #shared_name<'a> {
                    #[inline(always)]
                    pub unsafe fn new(priority: &'a Priority) -> Self {
                        #shared_name { priority }
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
                &shared_name,
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

    let manual = "Manual Codegen".to_string();

    let to_gen = quote! {

        pub struct __rtic_internal_fooShared {
            a: &'static mut u32,
            b: &'static mut i64,
        }


        impl __rtic_internal_fooShared {
        #[inline(always)]
        pub unsafe fn new() -> Self {
            __rtic_internal_fooShared {
                a: &mut *__rtic_internal_shared_resource_a
                .get_mut_unchecked()
                .as_mut_ptr(),
                b: &mut *__rtic_internal_shared_resource_b
                .get_mut_unchecked()
                .as_mut_ptr(),
                }
            }
        }

        // #[doc = #manual]
        // impl<'a> __rtic_internal_fooSharedResources<'a> {
        //     #[inline(always)]
        //     pub unsafe fn priority(&self) -> &rtic::export::Priority {
        //         self.priority
        //     }
        // }

        #[doc = #manual]
        impl<'a> rtic::Mutex for __rtic_internal_fooSharedResources<'a> {
            type T = __rtic_internal_fooShared;
            #[inline(always)]
            fn lock<RTIC_INTERNAL_R>(
                &mut self,
                f: impl FnOnce(&mut __rtic_internal_fooShared) -> RTIC_INTERNAL_R,
            ) -> RTIC_INTERNAL_R {
                /// Priority ceiling
                const CEILING: u8 = 1u8;
                unsafe {
                    rtic::export::lock(
                        &mut __rtic_internal_fooShared::new(),
                        self.priority(),
                        CEILING,
                        lm3s6965::NVIC_PRIO_BITS,
                        f,
                    )
                }
            }
        }
    };

    mod_app.push(to_gen);

    (mod_app, mod_resources)
}
