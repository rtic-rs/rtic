use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use rtic_syntax::ast::App;
use syn::Index;

use crate::{analyze::Analysis, codegen::util};

/// Generates code that runs after `#[init]` returns
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // Initialize late resources
    if !analysis.late_resources.is_empty() {
        // BTreeSet wrapped in a vector
        for name in analysis.late_resources.first().unwrap() {
            let mangled_name = util::mark_internal_ident(&name);
            // If it's live
            let cfgs = app.late_resources[name].cfgs.clone();
            if analysis.locations.get(name).is_some() {
                stmts.push(quote!(
                    // We include the cfgs
                    #(#cfgs)*
                    // Late resource is a RacyCell<MaybeUninit<T>>
                    // - `get_mut_unchecked` to obtain `MaybeUninit<T>`
                    // - `as_mut_ptr` to obtain a raw pointer to `MaybeUninit<T>`
                    // - `write` the defined value for the late resource T
                #mangled_name.get_mut_unchecked().as_mut_ptr().write(late.#name);
                ));
            }
        }
    }

    for (i, (monotonic, _)) in app.monotonics.iter().enumerate() {
        let idx = Index {
            index: i as u32,
            span: Span::call_site(),
        };
        stmts.push(quote!(monotonics.#idx.reset();));

        // Store the monotonic
        let name = util::monotonic_ident(&monotonic.to_string());
        let name = util::mark_internal_ident(&name);
        stmts.push(quote!(#name = Some(monotonics.#idx);));
    }

    // Enable the interrupts -- this completes the `init`-ialization phase
    stmts.push(quote!(rtic::export::interrupt::enable();));

    stmts
}
