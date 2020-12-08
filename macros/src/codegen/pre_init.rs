use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates code that runs before `#[init]`
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // Disable interrupts -- `init` must run with interrupts disabled
    stmts.push(quote!(rtic::export::interrupt::disable();));

    // Populate the FreeQueue
    for (name, task) in &app.software_tasks {
        let cap = task.args.capacity;
        let fq_ident = util::fq_ident(name);

        stmts.push(quote!(
            (0..#cap).for_each(|i| #fq_ident.enqueue_unchecked(i));
        ));
    }

    stmts.push(quote!(
        // To set the variable in cortex_m so the peripherals cannot be taken multiple times
        let mut core: rtic::export::Peripherals = rtic::export::Peripherals::steal().into();
    ));

    let device = &extra.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);

    let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

    // Unmask interrupts and set their priorities
    for (&priority, name) in interrupt_ids.chain(app.hardware_tasks.values().flat_map(|task| {
        if !util::is_exception(&task.args.binds) {
            Some((&task.args.priority, &task.args.binds))
        } else {
            // We do exceptions in another pass
            None
        }
    })) {
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

        // NOTE this also checks that the interrupt exists in the `Interrupt` enumeration
        let interrupt = util::interrupt_ident();
        stmts.push(quote!(
            core.NVIC.set_priority(
                you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml::#interrupt::#name,
                rtic::export::logical2hw(#priority, #nvic_prio_bits),
            );
        ));

        // NOTE unmask the interrupt *after* setting its priority: changing the priority of a pended
        // interrupt is implementation defined
        stmts.push(quote!(rtic::export::NVIC::unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml::#interrupt::#name);));
    }

    // Set exception priorities
    for (name, priority) in app.hardware_tasks.values().filter_map(|task| {
        if util::is_exception(&task.args.binds) {
            Some((&task.args.binds, task.args.priority))
        } else {
            None
        }
    }) {
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

        stmts.push(quote!(core.SCB.set_priority(
            rtic::export::SystemHandler::#name,
            rtic::export::logical2hw(#priority, #nvic_prio_bits),
        );));
    }

    // TODO: Update for noew monotonic
    // // Initialize the SysTick if there exist a TimerQueue
    // if extra.monotonic.is_some() {
    //     let priority = analysis.channels.keys().max().unwrap();

    //     // Compile time assert that this priority is supported by the device
    //     stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

    //     stmts.push(quote!(core.SCB.set_priority(
    //         rtic::export::SystemHandler::SysTick,
    //         rtic::export::logical2hw(#priority, #nvic_prio_bits),
    //     );));

    //     stmts.push(quote!(
    //         core.SYST.set_clock_source(rtic::export::SystClkSource::Core);
    //         core.SYST.enable_counter();
    //         core.DCB.enable_trace();
    //     ));
    // }

    // If there's no user `#[idle]` then optimize returning from interrupt handlers
    if app.idles.is_empty() {
        // Set SLEEPONEXIT bit to enter sleep mode when returning from ISR
        stmts.push(quote!(core.SCB.scr.modify(|r| r | 1 << 1);));
    }

    stmts
}
