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
    codegen::util,
    // codegen::util,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{parse, Attribute, Ident};

/// This macro implements the [`rtic::Mutex`] trait using the PLIC threshold for shared resources.
///
/// # Some remarks
///
/// If you use a threshold-based approach, you can adapt it using this as inspiration. You can also
/// have a look to the original cortex-m-basepri codegen binding. As far as we know, here you only
/// need to know **HOW MANY BITS YOUR PLATFORM USES TO REPRESENT PRIORITIES**.
/// The [`riscv::peripheral::plic::PriorityLevel`] trait stores this information in its
/// `N_PRIORITY_BITS` const generic.
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
                    riscv_slic::lock(#ptr, CEILING, f);
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

pub fn extra_modules(app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // generate code for the riscv e310x backend, parsing the
    // app arguments to dynamically generate the required dispatchers
    // in the SLIC implementation.
    let hw_slice: Vec<_> = app
        .hardware_tasks
        .values()
        .map(|task| &task.args.binds)
        .collect();
    let sw_slice: Vec<_> = app.args.dispatchers.keys().collect();
    let device = &app.args.device;
    let slic_module = quote!(riscv_slic::codegen!(#device, [#(#hw_slice,)*], [#(#sw_slice,)*]););
    stmts.push(slic_module);
    stmts
}

/// This macro is used to check at run-time that all the interruption dispatchers exist.
/// Probably, this macro fits in any architecture.
pub fn pre_init_checks(app: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    stmts.push(quote!(slic::clear_interrupts();));

    // check that all dispatchers exists in the `Interrupt` enumeration
    //let interrupt = util::interrupt_ident();
    // let rt_err = util::rt_err_ident();

    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = slic::Interrupt::#name;));
    }

    stmts
}

pub fn pre_init_enable_interrupts(_app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    let interrupt = util::interrupt_ident();
    // let rt_err = util::rt_err_ident();

    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

    // Set interrupt priorities and unmask them
    for (&p, name) in interrupt_ids {
        stmts.push(quote!(
            riscv_slic::set_priority(#interrupt::#name, #p);
        ));
    }
    //stmts.push(quote!(slic::set_interrupts();));
    stmts
}

pub fn architecture_specific_analysis(app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    // Check that there are enough external interrupts to dispatch the software tasks and the timer
    // queue handler
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
/// In most of the cases, this will be empty.
pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}

/// Macro to add statements to be executed at the end of all the interrupt handlers.
/// In most of the cases, this will be empty.
pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}
