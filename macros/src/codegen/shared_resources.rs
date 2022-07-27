use crate::{analyze::Analysis, check::Extra, codegen::util};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{analyze::Ownership, ast::App};
use std::collections::HashMap;

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

        let attrs = &res.attrs;

        // late resources in `util::link_section_uninit`
        // unless user specifies custom link section
        let section = if attrs.iter().any(|attr| attr.path.is_ident("link_section")) {
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
                Some(Ownership::Owned { priority } | Ownership::CoOwned { priority }) => *priority,
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
            use rtic::export::Priority;

            #(#mod_resources)*
        })
    };

    // Computing mapping of used interrupts to masks
    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

    let mut prio_to_masks = HashMap::new();
    let device = &extra.device;
    let mut uses_exceptions_with_resources = false;

    let mut mask_ids = Vec::new();

    for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().flat_map(|task| {
        if !util::is_exception(&task.args.binds) {
            Some((&task.args.priority, &task.args.binds))
        } else {
            // If any resource to the exception uses non-lock-free or non-local resources this is
            // not allwed on thumbv6.
            uses_exceptions_with_resources = uses_exceptions_with_resources
                || task
                    .args
                    .shared_resources
                    .iter()
                    .map(|(ident, access)| {
                        if access.is_exclusive() {
                            if let Some(r) = app.shared_resources.get(ident) {
                                !r.properties.lock_free
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .any(|v| v);

            None
        }
    })) {
        let v = prio_to_masks.entry(priority - 1).or_insert(Vec::new());
        v.push(quote!(#device::Interrupt::#name as u32));
        mask_ids.push(quote!(#device::Interrupt::#name as u32));
    }

    // Call rtic::export::create_mask([Mask; N]), where the array is the list of shifts

    let mut mask_arr = Vec::new();
    // NOTE: 0..3 assumes max 4 priority levels according to M0, M23 spec
    for i in 0..3 {
        let v = if let Some(v) = prio_to_masks.get(&i) {
            v.clone()
        } else {
            Vec::new()
        };

        mask_arr.push(quote!(
            rtic::export::create_mask([#(#v),*])
        ));
    }

    // Generate a constant for the number of chunks needed by Mask.
    let chunks_name = util::priority_mask_chunks_ident();
    mod_app.push(quote!(
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #chunks_name: usize = rtic::export::compute_mask_chunks([#(#mask_ids),*]);
    ));

    let masks_name = util::priority_masks_ident();
    mod_app.push(quote!(
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #masks_name: [rtic::export::Mask<#chunks_name>; 3] = [#(#mask_arr),*];
    ));

    if uses_exceptions_with_resources {
        mod_app.push(quote!(
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            const __rtic_internal_V6_ERROR: () = rtic::export::no_basepri_panic();
        ));
    }

    (mod_app, mod_resources)
}
