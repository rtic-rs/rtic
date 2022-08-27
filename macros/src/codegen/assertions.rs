use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{analyze::Analysis, check::Extra, codegen::util};
use rtic_syntax::ast::App;

/// Generates compile-time assertions that check that types implement the `Send` / `Sync` traits
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
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

    let device = &extra.device;
    let chunks_name = util::priority_mask_chunks_ident();
    let no_basepri_checks: Vec<_> = app
        .hardware_tasks
        .iter()
        .filter_map(|(_, task)| {
            if !util::is_exception(&task.args.binds) {
                let interrupt_name = &task.args.binds;
                Some(quote!(
                    if (#device::Interrupt::#interrupt_name as usize) >= (#chunks_name * 32) {
                        ::core::panic!("An interrupt out of range is used while in armv6 or armv8m.base");
                    }
                ))
            } else {
                None
            }
        })
        .collect();

    let const_check = quote! {
        const _CONST_CHECK: () = {
            if !rtic::export::have_basepri() {
                #(#no_basepri_checks)*
            } else {
                // TODO: Add armv7 checks here
            }
        };

        let _ = _CONST_CHECK;
    };

    stmts.push(const_check);

    stmts
}
