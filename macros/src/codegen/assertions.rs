use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::analyze::Analysis;
use rtic_syntax::ast::App;

/// Generates compile-time assertions that check that types implement the `Send` / `Sync` traits
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    for ty in &analysis.send_types {
        stmts.push(quote!(rtic::export::assert_send::<#ty>();));
    }

    for ty in &analysis.sync_types {
        stmts.push(quote!(rtic::export::assert_sync::<#ty>();));
    }

    for (_, monotonic) in &app.monotonics {
        let ty = &monotonic.ty;
        stmts.push(quote!(rtic::export::assert_monotonic::<#ty>();));
    }

    stmts
}
