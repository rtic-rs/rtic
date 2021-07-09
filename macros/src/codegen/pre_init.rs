use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates code that runs before `#[init]`
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    let rt_err = util::rt_err_ident();

    // Disable interrupts -- `init` must run with interrupts disabled
    stmts.push(quote!(rtic::export::interrupt::disable();));

    // Populate the FreeQueue
    for (name, task) in &app.software_tasks {
        let cap = task.args.capacity;
        let fq_ident = util::fq_ident(name);
        let fq_ident = util::mark_internal_ident(&fq_ident);

        stmts.push(quote!(
            (0..#cap).for_each(|i| #fq_ident.get_mut_unchecked().enqueue_unchecked(i));
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
        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

        stmts.push(quote!(core.SCB.set_priority(
            rtic::export::SystemHandler::#name,
            rtic::export::logical2hw(#priority, #nvic_prio_bits),
        );));
    }

    // Initialize monotonic's interrupts and timer queues
    for (_, monotonic) in &app.monotonics {
        let priority = &monotonic.args.priority;
        let binds = &monotonic.args.binds;
        let monotonic_name = monotonic.ident.to_string();
        let tq = util::tq_ident(&monotonic_name);
        let tq = util::mark_internal_ident(&tq);

        // Initialize timer queues
        stmts.push(
            quote!(#tq.get_mut_unchecked().as_mut_ptr().write(rtic::export::TimerQueue::new());),
        );

        // Compile time assert that this priority is supported by the device
        stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

        let mono_type = &monotonic.ty;

        if &*binds.to_string() == "SysTick" {
            stmts.push(quote!(
                core.SCB.set_priority(
                    rtic::export::SystemHandler::SysTick,
                    rtic::export::logical2hw(#priority, #nvic_prio_bits),
                );

                // Always enable monotonic interrupts if they should never be off
                if !<#mono_type as rtic::Monotonic>::DISABLE_INTERRUPT_ON_EMPTY_QUEUE {
                    core::mem::transmute::<_, cortex_m::peripheral::SYST>(())
                        .enable_interrupt();
                }
            ));
        } else {
            // NOTE this also checks that the interrupt exists in the `Interrupt` enumeration
            let interrupt = util::interrupt_ident();
            stmts.push(quote!(
                core.NVIC.set_priority(
                    #rt_err::#interrupt::#binds,
                    rtic::export::logical2hw(#priority, #nvic_prio_bits),
                );

                // Always enable monotonic interrupts if they should never be off
                if !<#mono_type as rtic::Monotonic>::DISABLE_INTERRUPT_ON_EMPTY_QUEUE {
                    rtic::export::NVIC::unmask(#rt_err::#interrupt::#binds);
                }
            ));
        }
    }

    // If there's no user `#[idle]` then optimize returning from interrupt handlers
    if app.idle.is_none() {
        // Set SLEEPONEXIT bit to enter sleep mode when returning from ISR
        stmts.push(quote!(core.SCB.scr.modify(|r| r | 1 << 1);));
    }

    stmts
}
