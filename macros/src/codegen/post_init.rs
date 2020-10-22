use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, codegen::util};

/// Generates code that runs after `#[init]` returns
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // Initialize late resources
    if !analysis.late_resources.is_empty() {
        // BTreeSet wrapped in a vector
        for name in analysis.late_resources.first().unwrap() {
            let mangled_name = util::mangle_ident(&name);
            // If it's live
            let cfgs = app.late_resources[name].cfgs.clone();
            if analysis.locations.get(name).is_some() {
                // Need to also include the cfgs
                stmts.push(quote!(
                #(#cfgs)*
                #mangled_name.as_mut_ptr().write(late.#name);
                ));
            }
        }
    }

    // Enable the interrupts -- this completes the `init`-ialization phase
    stmts.push(quote!(rtic::export::interrupt::enable();));

    stmts
}
