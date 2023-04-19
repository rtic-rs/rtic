//! Proof of concept: porting RTIC for the RISC-V chips with PLIC.
//!
//! # Related crates
//!
//! - riscv
//! - riscv-rt
//! - riscv-slic
//!
//! # Some clarifications
//!
//! This implementation will use a generic software interrupt implementation designed for risc-v processors.
//! The implementation of the sw interrupts themselves is handled per-processor, depending on their workflow.
//! In the case of our example board, e310x, the CLINT peripheral takes care of handling them.

use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{parse, Attribute, Ident};

/// Utility function to get the SLIC interrupt module.
pub fn interrupt_mod_ident() -> TokenStream2 {
    syn::parse_str("slic::Interrupt").unwrap()
}

/// This macro implements the [`rtic::Mutex`] trait for shared resources using the SLIC.
pub fn impl_mutex(
    _app: &App,
    _analysis: &CodegenAnalysis,
    cfgs: &[Attribute],
    resources_prefix: bool,
    name: &Ident,
    ty: &TokenStream2,
    ceiling: u8,
    ptr: &TokenStream2,
) -> TokenStream2 {
    let path = if resources_prefix {
        quote!(shared_resources::#name)
    } else {
        quote!(#name)
    };

    quote!(
        #(#cfgs)*
        impl<'a> rtic::Mutex for #path<'a> {
            type T = #ty;

            #[inline(always)]
            fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {

                const CEILING: u16 = #ceiling.into();

                unsafe {
                    rtic::export::lock(#ptr, CEILING, f);
                }
            }
        }
    )
}

/// This macro is used to define additional compile-time assertions in case the platform needs it.
/// The Cortex-M implementations do not use it. Thus, we think we do not need it either.
pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

/// The SLIC requires us to call to the [`riscv_rtic::codegen`] macro to generate
/// the appropriate SLIC structure, interrupt enumerations, etc.
pub fn extra_modules(app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    let hw_slice: Vec<_> = app
        .hardware_tasks
        .values()
        .map(|task| &task.args.binds)
        .collect();
    let sw_slice: Vec<_> = app.args.dispatchers.keys().collect();
    let device = &app.args.device;

    stmts.push(quote!(rtic::export::codegen!(#device, [#(#hw_slice,)*], [#(#sw_slice,)*]);));
    stmts
}

/// This macro is used to check at run-time that all the interruption dispatchers exist.
pub fn pre_init_checks(app: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // check that all dispatchers exists in the `slic::Interrupt` enumeration
    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = slic::Interrupt::#name;));
    }

    stmts
}

/// This macro must perform all the required operations to activate the
/// interrupt sources with their corresponding priority level.
pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // First, we reset and disable all the interrupt controllers
    stmts.push(quote!(rtic::export::clear_interrupts();));

    // Then, we set the corresponding priorities
    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));
    for (&p, name) in interrupt_ids.chain(
        app.hardware_tasks
            .values()
            .map(|task| (&task.args.priority, &task.args.binds)),
    ) {
        stmts.push(quote!(
            rtic::export::set_priority(slic::Interrupt::#name, #p);
        ));
    }
    // Finally, we activate the interrupts
    stmts.push(quote!(rtic::export::set_interrupts();));
    stmts
}

/// Any additional checks that depend on the system architecture.
pub fn architecture_specific_analysis(app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    // Check that there are enough external interrupts to dispatch the software tasks and the timer queue handler
    let mut first = None;
    let priorities = app
        .software_tasks
        .iter()
        .map(|(name, task)| {
            first = Some(name);
            task.args.priority
        })
        .filter(|prio| *prio > 0)
        .collect::<HashSet<_>>();

    let need = priorities.len();
    let given = app.args.dispatchers.len();
    if need > given {
        let s = {
            format!(
                "not enough interrupts to dispatch \
                    all software tasks (need: {need}; given: {given})"
            )
        };

        return Err(parse::Error::new(first.unwrap().span(), s));
    }

    Ok(()) // TODO
}

/// Macro to add statements to be executed at the beginning of all the interrupt handlers.
pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

/// Macro to add statements to be executed at the end of all the interrupt handlers.
pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}
