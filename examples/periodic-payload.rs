// # Pointers (old)
//
// ~52~ 48 bytes .bss
//
// # -Os
//
// init
// a(bl=8000000, now=8000180, input=0)
// a(bl=16000000, now=16000180, input=1)
// a(bl=24000000, now=24000180, input=2)
//
// # -O3
//
// init
// a(bl=8000000, now=8000168, input=0)
// a(bl=16000000, now=16000168, input=1)
// a(bl=24000000, now=24000168, input=2)
//
// # Indices (new)
//
// 32 bytes .bss
//
// ## -O3
//
// init
// a(bl=8000000, now=8000164, input=0)
// a(bl=16000000, now=16000164, input=1)
//
// ## -Os
//
// init
// a(bl=8000000, now=8000179, input=0)
// a(bl=16000000, now=16000179, input=1)

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
use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static ITM: ITM;
    },

    init: {
        async_after: [a],
    },

    free_interrupts: [EXTI0],

    tasks: {
        a: {
            async_after: [a],
            input: u16,
            resources: [ITM],
        },
    },
}

const MS: u32 = 8_000;
const S: u32 = 1_000 * MS;

#[inline(always)]
fn init(mut ctxt: init::Context) -> init::LateResources {
    iprintln!(&mut ctxt.core.ITM.stim[0], "init");

    ctxt.async.a.post(&mut ctxt.threshold, 1 * S, 0).ok();

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
    let itm = ctxt.resources.ITM;
    iprintln!(
        &mut itm.stim[0],
        "a(bl={}, now={}, input={})",
        bl,
        now,
        input
    );

    ctxt.async
        .a
        .post(&mut ctxt.threshold, 1 * S, input + 1)
        .ok();
}
