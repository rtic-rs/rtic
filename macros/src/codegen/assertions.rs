use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::analyze::Analysis;

/// Generates compile-time assertions that check that types implement the `Send` / `Sync` traits
pub fn codegen(analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // we don't generate *all* assertions on all cores because the user could conditionally import a
    // type only on some core (e.g. `#[cfg(core = "0")] use some::Type;`)

    //if let Some(types) = analysis.send_types {
    for ty in &analysis.send_types {
        stmts.push(quote!(rtic::export::assert_send::<#ty>();));
    }
    //}

    //if let Some(types) = analysis.sync_types {
    for ty in &analysis.sync_types {
        stmts.push(quote!(rtic::export::assert_sync::<#ty>();));
    }
    //}

    // if the `schedule` API is used in more than one core then we need to check that the
    // `monotonic` timer can be used in multi-core context
    /*
    if analysis.timer_queues.len() > 1 && analysis.timer_queues.contains_key(&core) {
        let monotonic = extra.monotonic();
        stmts.push(quote!(rtic::export::assert_multicore::<#monotonic>();));
    }
    */

    stmts
}
