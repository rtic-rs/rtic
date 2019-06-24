use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{analyze::Analysis, check::Extra};

/// Generates compile-time assertions that check that types implement the `Send` / `Sync` traits
pub fn codegen(core: u8, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
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

    // if the `schedule` API is used in more than one core then we need to check that the
    // `monotonic` timer can be used in multi-core context
    if analysis.timer_queues.len() > 1 && analysis.timer_queues.contains_key(&core) {
        let monotonic = extra.monotonic();
        stmts.push(quote!(rtfm::export::assert_multicore::<#monotonic>();));
    }

    stmts
}
