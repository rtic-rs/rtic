// # Pointers (old)
//
// ~40~ 32 bytes .bss
//
// # Indices (new)
//
// 12 bytes .bss

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use cortex_m::asm;
use rtfm::app;

app! {
    device: stm32f103xx,

    init: {
        async: [a],
    },

    free_interrupts: [EXTI1],

    tasks: {
        exti0: {
            interrupt: EXTI0,
            async: [a],
        },

        a: {},
    },
}

#[inline(always)]
fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

#[inline(always)]
fn idle(_ctxt: idle::Context) -> ! {
    loop {
        asm::wfi();
    }
}

fn exti0(mut ctxt: exti0::Context) {
    ctxt.async.a.post(&mut ctxt.threshold, ()).ok();
}

fn a(_ctxt: a::Context) {
    asm::bkpt();
}
