use super::bindings::{pre_init_checks, pre_init_enable_interrupts};
use crate::analyze::Analysis;
use crate::syntax::ast::App;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates code that runs before `#[init]`
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // FIXME: temporary fix for riscv
    // Disable interrupts -- `init` must run with interrupts disabled
    #[cfg(not(any(riscv_slic_backend)))]
    stmts.push(quote!(rtic::export::interrupt::disable();));

    #[cfg(not(any(riscv_slic_backend)))]
    stmts.push(quote!(
        // To set the variable in cortex_m so the peripherals cannot be taken multiple times
        let mut core: rtic::export::Peripherals = rtic::export::Peripherals::steal().into();
    ));

    stmts.append(&mut pre_init_checks(app, analysis));

    stmts.append(&mut pre_init_enable_interrupts(app, analysis));

    stmts
}
