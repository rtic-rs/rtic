use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use rtic_syntax::ast::App;
use syn::Index;

use crate::{analyze::Analysis, codegen::util};

/// Generates code that runs after `#[init]` returns
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // Initialize shared resources
    for (name, res) in &app.shared_resources {
        let mangled_name = util::static_shared_resource_ident(name);
        // If it's live
        let cfgs = res.cfgs.clone();
        if analysis.shared_resource_locations.get(name).is_some() {
            stmts.push(quote!(
                // We include the cfgs
                #(#cfgs)*
                // Resource is a RacyCell<MaybeUninit<T>>
                // - `get_mut_unchecked` to obtain `MaybeUninit<T>`
                // - `as_mut_ptr` to obtain a raw pointer to `MaybeUninit<T>`
                // - `write` the defined value for the late resource T
                #mangled_name.get_mut_unchecked().as_mut_ptr().write(shared_resources.#name);
            ));
        }
    }

    // Initialize local resources
    for (name, res) in &app.local_resources {
        let mangled_name = util::static_local_resource_ident(name);
        // If it's live
        let cfgs = res.cfgs.clone();
        if analysis.local_resource_locations.get(name).is_some() {
            stmts.push(quote!(
                // We include the cfgs
                #(#cfgs)*
                // Resource is a RacyCell<MaybeUninit<T>>
                // - `get_mut_unchecked` to obtain `MaybeUninit<T>`
                // - `as_mut_ptr` to obtain a raw pointer to `MaybeUninit<T>`
                // - `write` the defined value for the late resource T
                #mangled_name.get_mut_unchecked().as_mut_ptr().write(local_resources.#name);
            ));
        }
    }

    for (i, (monotonic, _)) in app.monotonics.iter().enumerate() {
        // For future use
        // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
        // stmts.push(quote!(#[doc = #doc]));

        let idx = Index {
            index: i as u32,
            span: Span::call_site(),
        };
        stmts.push(quote!(monotonics.#idx.reset();));

        // Store the monotonic
        let name = util::monotonic_ident(&monotonic.to_string());
        stmts.push(quote!(*#name.get_mut_unchecked() = Some(monotonics.#idx);));
    }

    // Enable the interrupts -- this completes the `init`-ialization phase
    stmts.push(quote!(rtic::export::interrupt::enable();));

    stmts
}
