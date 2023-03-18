//! Proof of concept: porting RTIC for the E310x RISC-V chip.
//!
//! # Related crates
//!
//! - riscv
//! - riscv-rt
//! - e310x
//! - e310x-hal
//!
//! # Some clarifications
//!
//! This implementation uses CLINT and PLIC jointly. As PLIC allows interrupt
//! thresholds, we are using the cortex-m-basepri as inspiration.
//! We will document all our decisions for easing future ports.

use crate::{
    analyze::Analysis as CodegenAnalysis,
    codegen::util,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse, Attribute, Ident};

// Set global to reuse it
const E310X_PRIO_BITS: u8 = 3; // TODO we won't use this once we standardize the priority bits thing in RISC-V

/// This macro implements the [`rtic::Mutex`] trait using the PLIC threshold for shared resources.
///
/// # Some remarks
///
/// If you use a threshold-based approach, you can adapt it using this as inspiration. You can also
/// have a look to the original cortex-m-basepri codegen binding. As far as we know, here you only
/// need to know **HOW MANY BITS YOUR PLATFORM USE TO REPRESENT PRIORITIES**.
/// As the PAC of our platform does not include this information, we "hardcoded" it to 3.
///
/// # Future work
///
/// We should come up with a more standard mechanism for this.
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

    // E310x supports interrupt levels from 0 to 7. As future work, we should try
    // to standardize defining the priority bits of each microcontroller in each PAC
    // let _device = &app.args.device; // TODO we will use this once we standardize the priority bits thing in RISC-V
    quote!(
        #(#cfgs)*
        impl<'a> rtic::Mutex for #path<'a> {
            type T = #ty;

            #[inline(always)]
            fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {

                const CEILING: u8 = #ceiling;

                unsafe {
                    rtic::export::lock(
                        #ptr,
                        CEILING,
                        E310X_PRIO_BITS,  // TODO we will use this once we standardize the priority bits thing in RISC-V
                        f,
                    )
                }
            }
        }
    )
}

/// This macro is used to define additional compile-time assertions in case the platform needs it.
/// The Cortex-M implementations do not use it. Thus, we think we do not need it either.
pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}

/// This macro is used to check at run-time that all the interruption dispatchers exist.
/// Probably, this macro fits in any architecture.
pub fn pre_init_checks(app: &App, _: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // check that all dispatchers exists in the `Interrupt` enumeration
    let interrupt = util::interrupt_ident();
    let rt_err = util::rt_err_ident();

    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = #rt_err::#interrupt::#name;));
    }

    stmts
}

pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    // Take the implementation from cortex as it's mostly similar
    let mut stmts = vec![];

    let interrupt = util::interrupt_ident();
    let rt_err = util::rt_err_ident();
    let device = &app.args.device;
    let prio_bits = quote!(E310X_PRIO_BITS);
    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

    // Unmask interrupts and set their priorities
    for (&priority, name) in interrupt_ids {
        let es = format!(
            "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
        );
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(
            const _: () =  if (1 << E310X_PRIO_BITS) < #priority as usize { ::core::panic!(#es); };
        ));
        // TODO: what does core reference??
        stmts.push(quote!(
            core.plic.INTERRUPT.set_priority(
                #rt_err::#interrupt::#name,
                rtic::export::cortex_logical2hw(#priority, #nvic_prio_bits),
            );
        ));

        // NOTE unmask the interrupt *after* setting its priority: changing the priority of a pended
        // interrupt is implementation defined
        stmts.push(quote!(rtic::export::NVIC::unmask(#rt_err::#interrupt::#name);));
    }

    vec![] // TODO
}

pub fn architecture_specific_analysis(_app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
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
