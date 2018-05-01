#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use cortex_m::asm;
use cortex_m::peripheral::{DWT, ITM};
use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static ITM: ITM;
    },

    free_interrupts: [EXTI1],

    tasks: {
        exti0: {
            interrupt: EXTI0,
            async_after: [a],
            resources: [ITM],
        },

        a: {
            resources: [ITM],
        },
    },
}

const S: u32 = 8_000_000;

#[inline(always)]
fn init(ctxt: init::Context) -> init::LateResources {
    unsafe { rtfm::set_pending(stm32f103xx::Interrupt::EXTI0) }

    init::LateResources { ITM: ctxt.core.ITM }
}

#[inline(always)]
fn idle(_ctxt: idle::Context) -> ! {
    loop {
        asm::wfi();
    }
}

fn exti0(mut ctxt: exti0::Context) {
    let now = DWT::get_cycle_count();
    iprintln!(
        &mut ctxt.resources.ITM.stim[0],
        "exti0(bl={}, now={})",
        ctxt.baseline,
        now
    );

    let t = &mut ctxt.threshold;
    ctxt.async.a.post(t, 1 * S, ()).ok();
}

fn a(ctxt: a::Context) {
    let now = DWT::get_cycle_count();
    iprintln!(
        &mut ctxt.resources.ITM.stim[0],
        "a(bl={}, now={})",
        ctxt.baseline,
        now
    );
}
