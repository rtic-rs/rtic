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

    use std::collections::HashMap;
    let mut masks: HashMap<u8, _> = std::collections::HashMap::new();
    let device = &extra.device;

    for p in 0..3 {
        masks.insert(p, quote!(0));
    }

    for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().flat_map(|task| {
        if !util::is_exception(&task.args.binds) {
            Some((&task.args.priority, &task.args.binds))
        } else {
            // TODO: exceptions not implemented
            None
        }
    })) {
        let name = quote!(#device::Interrupt::#name as u32);
        if let Some(v) = masks.get_mut(&(priority - 1)) {
            *v = quote!(#v | 1 << #name);
        };
    }

    let mut mask_arr: Vec<(_, _)> = masks.iter().collect();
    mask_arr.sort_by_key(|(k, _v)| *k);
    let mask_arr: Vec<_> = mask_arr.iter().map(|(_, v)| v).collect();

    mod_app.push(quote!(
        #[cfg(not(armv7m))]
        const MASKS: [u32; 3] = [#(#mask_arr),*];
    ));

    (mod_app, mod_resources)
}
