use crate::syntax::ast::App;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{analyze::Analysis, codegen::util};

/// Generates code that runs before `#[init]`
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    let rt_err = util::rt_err_ident();

    // Disable interrupts -- `init` must run with interrupts disabled
    stmts.push(quote!(rtic::export::interrupt::disable();));

    stmts.push(quote!(
        // To set the variable in cortex_m so the peripherals cannot be taken multiple times
        let mut core: rtic::export::Peripherals = rtic::export::Peripherals::steal().into();
    ));

    let device = &app.args.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);

    // check that all dispatchers exists in the `Interrupt` enumeration regardless of whether
    // they are used or not
    let interrupt = util::interrupt_ident();
    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = #rt_err::#interrupt::#name;));
    }

    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

    // Unmask interrupts and set their priorities
    for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().filter_map(|task| {
        if util::is_exception(&task.args.binds) {
            // We do exceptions in another pass
            None
        } else {
            Some((&task.args.priority, &task.args.binds))
        }
    })) {
        let es = format!(
            "Maximum priority used by interrupt vector '{}' is more than supported by hardware",
            name
        );
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(
            const _: () =  if (1 << #nvic_prio_bits) < #priority as usize { ::core::panic!(#es); };
        ));

        stmts.push(quote!(
            core.NVIC.set_priority(
                #rt_err::#interrupt::#name,
                rtic::export::logical2hw(#priority, #nvic_prio_bits),
            );
        ));

        // NOTE unmask the interrupt *after* setting its priority: changing the priority of a pended
        // interrupt is implementation defined
        stmts.push(quote!(rtic::export::NVIC::unmask(#rt_err::#interrupt::#name);));
    }

    // Set exception priorities
    for (name, priority) in app.hardware_tasks.values().filter_map(|task| {
        if util::is_exception(&task.args.binds) {
            Some((&task.args.binds, task.args.priority))
        } else {
            None
        }
    }) {
        let es = format!(
            "Maximum priority used by interrupt vector '{}' is more than supported by hardware",
            name
        );
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(
            const _: () =  if (1 << #nvic_prio_bits) < #priority as usize { ::core::panic!(#es); };
        ));

        stmts.push(quote!(core.SCB.set_priority(
            rtic::export::SystemHandler::#name,
            rtic::export::logical2hw(#priority, #nvic_prio_bits),
        );));
    }

    stmts
}
