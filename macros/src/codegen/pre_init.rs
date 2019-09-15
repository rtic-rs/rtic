use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates code that runs before `#[init]`
pub fn codegen(
    core: u8,
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // `const_app_pre_init` -- `static` variables for barriers
    Vec<TokenStream2>,
    // `pre_init_stmts`
    Vec<TokenStream2>,
) {
    let mut const_app = vec![];
    let mut stmts = vec![];

    // disable interrupts -- `init` must run with interrupts disabled
    stmts.push(quote!(rtfm::export::interrupt::disable();));

    // populate this core `FreeQueue`s
    for (name, senders) in &analysis.free_queues {
        let task = &app.software_tasks[name];
        let cap = task.args.capacity;

        for &sender in senders.keys() {
            if sender == core {
                let fq = util::fq_ident(name, sender);

                stmts.push(quote!(
                    (0..#cap).for_each(|i| #fq.enqueue_unchecked(i));
                ));
            }
        }
    }

    stmts.push(quote!(
        // NOTE(transmute) to avoid debug_assertion in multi-core mode
        let mut core: rtfm::export::Peripherals = core::mem::transmute(());
    ));

    let device = extra.device;
    let nvic_prio_bits = quote!(#device::NVIC_PRIO_BITS);

    // unmask interrupts and set their priorities
    for (&priority, name) in analysis
        .interrupts
        .get(&core)
        .iter()
        .flat_map(|interrupts| *interrupts)
        .chain(app.hardware_tasks.values().flat_map(|task| {
            if !util::is_exception(&task.args.binds) {
                Some((&task.args.priority, &task.args.binds))
            } else {
                // we do exceptions in another pass
                None
            }
        }))
    {
        // compile time assert that this priority is supported by the device
        stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

        // NOTE this also checks that the interrupt exists in the `Interrupt` enumeration
        let interrupt = util::interrupt_ident(core, app.args.cores);
        stmts.push(quote!(
            core.NVIC.set_priority(
                #device::#interrupt::#name,
                rtfm::export::logical2hw(#priority, #nvic_prio_bits),
            );
        ));

        // NOTE unmask the interrupt *after* setting its priority: changing the priority of a pended
        // interrupt is implementation defined
        stmts.push(quote!(rtfm::export::NVIC::unmask(#device::#interrupt::#name);));
    }

    // cross-spawn barriers: now that priorities have been set and the interrupts have been unmasked
    // we are ready to receive messages from *other* cores
    if analysis.spawn_barriers.contains_key(&core) {
        let sb = util::spawn_barrier(core);
        let shared = if cfg!(feature = "heterogeneous") {
            Some(quote!(
                #[rtfm::export::shared]
            ))
        } else {
            None
        };

        const_app.push(quote!(
            #shared
            static #sb: rtfm::export::Barrier = rtfm::export::Barrier::new();
        ));

        // unblock cores that may send us a message
        stmts.push(quote!(
            #sb.release();
        ));
    }

    // set exception priorities
    for (name, priority) in app.hardware_tasks.values().filter_map(|task| {
        if util::is_exception(&task.args.binds) {
            Some((&task.args.binds, task.args.priority))
        } else {
            None
        }
    }) {
        // compile time assert that this priority is supported by the device
        stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

        stmts.push(quote!(core.SCB.set_priority(
            rtfm::export::SystemHandler::#name,
            rtfm::export::logical2hw(#priority, #nvic_prio_bits),
        );));
    }

    // initialize the SysTick
    if let Some(tq) = analysis.timer_queues.get(&core) {
        let priority = tq.priority;

        // compile time assert that this priority is supported by the device
        stmts.push(quote!(let _ = [(); ((1 << #nvic_prio_bits) - #priority as usize)];));

        stmts.push(quote!(core.SCB.set_priority(
            rtfm::export::SystemHandler::SysTick,
            rtfm::export::logical2hw(#priority, #nvic_prio_bits),
        );));

        stmts.push(quote!(
            core.SYST.set_clock_source(rtfm::export::SystClkSource::Core);
            core.SYST.enable_counter();
            core.DCB.enable_trace();
        ));
    }

    // if there's no user `#[idle]` then optimize returning from interrupt handlers
    if app.idles.get(&core).is_none() {
        // Set SLEEPONEXIT bit to enter sleep mode when returning from ISR
        stmts.push(quote!(core.SCB.scr.modify(|r| r | 1 << 1);));
    }

    // cross-spawn barriers: wait until other cores are ready to receive messages
    for (&receiver, senders) in &analysis.spawn_barriers {
        // only block here if `init` can send messages to `receiver`
        if senders.get(&core) == Some(&true) {
            let sb = util::spawn_barrier(receiver);

            stmts.push(quote!(
                #sb.wait();
            ));
        }
    }

    (const_app, stmts)
}
