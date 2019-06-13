use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::analyze::Analysis;

/// Generates compile-time assertions that check that types implement the `Send` / `Sync` traits
pub fn codegen(core: u8, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // we don't generate *all* assertions on all cores because the user could conditionally import a
    // type only on some core (e.g. `#[cfg(core = "0")] use some::Type;`)

    if let Some(types) = analysis.send_types.get(&core) {
        for ty in types {
            stmts.push(quote!(rtfm::export::assert_send::<#ty>();));
        }
    }

    if let Some(types) = analysis.sync_types.get(&core) {
        for ty in types {
            stmts.push(quote!(rtfm::export::assert_sync::<#ty>();));
        }
    }

    stmts
}
