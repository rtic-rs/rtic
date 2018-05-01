// # -Os
// init
// a(bl=16000000, now=16000249)
// b(bl=24000000, now=24000248)
// a(bl=32000000, now=32000249)
// b(bl=48000000, now=48000282)
// a(bl=48000000, now=48001731)
// a(bl=64000000, now=64000249)
// b(bl=72000000, now=72000248)
// a(bl=80000000, now=80000249)
// b(bl=96000000, now=96000282)
// a(bl=96000000, now=96001731)

// # -O3
// init
// a(bl=16000000, now=16000228)
// b(bl=24000000, now=24000231)
// a(bl=32000000, now=32000228)
// b(bl=48000000, now=48000257)
// a(bl=48000000, now=48001705)
// a(bl=64000000, now=64000228)
// b(bl=72000000, now=72000231)
// a(bl=80000000, now=80000228)
// b(bl=96000000, now=96000257)
// a(bl=96000000, now=96001705)

#![allow(warnings)]
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
// extern crate panic_abort;
extern crate panic_itm;
extern crate stm32f103xx;

use cortex_m::asm;
use cortex_m::peripheral::{DWT, ITM};
use rtfm::{app, Resource};

app! {
    device: stm32f103xx,

    resources: {
        static ITM: ITM;
    },

    init: {
        async_after: [a, b],
    },

    free_interrupts: [EXTI0, EXTI1],

    tasks: {
        a: {
            async_after: [a],
            resources: [ITM],
        },

        b: {
            async_after: [b],
            priority: 2,
            resources: [ITM],
        },
    },
}

const MS: u32 = 8_000;
const S: u32 = 1_000 * MS;

#[inline(always)]
fn init(mut ctxt: init::Context) -> init::LateResources {
    iprintln!(&mut ctxt.core.ITM.stim[0], "init");

    ctxt.async.a.post(&mut ctxt.threshold, 2 * S, ()).ok();
    ctxt.async.b.post(&mut ctxt.threshold, 3 * S, ()).ok();

    init::LateResources { ITM: ctxt.core.ITM }
}

#[inline(always)]
fn idle(_ctxt: idle::Context) -> ! {
    loop {
        asm::wfi();
    }
}

fn a(mut ctxt: a::Context) {
    let now = DWT::get_cycle_count();

    let bl = ctxt.baseline;
    ctxt.resources.ITM.claim_mut(&mut ctxt.threshold, |itm, _| {
        iprintln!(&mut itm.stim[0], "a(bl={}, now={})", bl, now);
    });

    ctxt.async.a.post(&mut ctxt.threshold, 2 * S, ()).ok();
}

fn b(mut ctxt: b::Context) {
    let now = DWT::get_cycle_count();

    let bl = ctxt.baseline;
    let t = &mut ctxt.threshold;
    iprintln!(
        &mut ctxt.resources.ITM.borrow_mut(t).stim[0],
        "b(bl={}, now={})",
        bl,
        now
    );

    ctxt.async.b.post(t, 3 * S, ()).ok();
}
