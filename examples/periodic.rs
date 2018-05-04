// 52 bytes .bss
//
// # -Os
// init
// a(bl=8000000, now=8000180)
// a(bl=16000000, now=16000180)
//
// # -O3
// a(bl=8000000, now=8000168)
// a(bl=16000000, now=16000168)

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
            resources: [ITM],
        },
    },
}

const MS: u32 = 8_000;
const S: u32 = 1_000 * MS;

#[inline(always)]
fn init(mut ctxt: init::Context) -> init::LateResources {
    iprintln!(&mut ctxt.core.ITM.stim[0], "init");

    ctxt.async.a.post(&mut ctxt.threshold, 1 * S, ()).ok();

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
    let itm = ctxt.resources.ITM;
    iprintln!(&mut itm.stim[0], "a(bl={}, now={})", bl, now);

    ctxt.async.a.post(&mut ctxt.threshold, 1 * S, ()).ok();
}
