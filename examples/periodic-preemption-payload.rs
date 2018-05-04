// 104 bytes .bss
//
// # -Os
// a(bl=16000000, now=16000248, input=0)
// b(bl=24000000, now=24000251, input=0)
// a(bl=32000000, now=32000248, input=1)
// b(bl=48000000, now=48000283, input=1)
// a(bl=48000000, now=48002427, input=2)
// a(bl=64000000, now=64000248, input=3)
// b(bl=72000000, now=72000251, input=2)
// a(bl=80000000, now=80000248, input=4)
// b(bl=96000000, now=96000283, input=3)
// a(bl=96000000, now=96002427, input=5)

// # -O3
// init
// a(bl=16000000, now=16000231, input=0)
// b(bl=24000000, now=24000230, input=0)
// a(bl=32000000, now=32000231, input=1)
// b(bl=48000000, now=48000259, input=1)
// a(bl=48000000, now=48002397, input=2)
// a(bl=64000000, now=64000231, input=3)
// b(bl=72000000, now=72000230, input=2)
// a(bl=80000000, now=80000231, input=4)
// b(bl=96000000, now=96000259, input=3)
// a(bl=96000000, now=96002397, input=5)

#![deny(unsafe_code)]
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
            input: u32,
            resources: [ITM],
        },

        b: {
            async_after: [b],
            input: u32,
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

    ctxt.async.a.post(&mut ctxt.threshold, 2 * S, 0).ok();
    ctxt.async.b.post(&mut ctxt.threshold, 3 * S, 0).ok();

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

    let input = ctxt.input;
    let bl = ctxt.baseline;
    ctxt.resources.ITM.claim_mut(&mut ctxt.threshold, |itm, _| {
        iprintln!(
            &mut itm.stim[0],
            "a(bl={}, now={}, input={})",
            bl,
            now,
            input
        );
    });

    ctxt.async
        .a
        .post(&mut ctxt.threshold, 2 * S, input + 1)
        .ok();
}

fn b(mut ctxt: b::Context) {
    let now = DWT::get_cycle_count();

    let bl = ctxt.baseline;
    let input = ctxt.input;
    let t = &mut ctxt.threshold;
    iprintln!(
        &mut ctxt.resources.ITM.borrow_mut(t).stim[0],
        "b(bl={}, now={}, input={})",
        bl,
        now,
        input,
    );

    ctxt.async.b.post(t, 3 * S, input + 1).ok();
}
