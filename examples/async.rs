#![allow(warnings)]
// #![deny(unsafe_code)]
// #![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
// extern crate panic_abort;
extern crate panic_itm;
extern crate stm32f103xx;

use core::mem;

use cortex_m::asm;
use cortex_m::peripheral::{DWT, ITM};
use rtfm::{app, Resource};

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
fn init(mut ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

#[inline(always)]
fn idle(ctxt: idle::Context) -> ! {
    loop {
        asm::wfi();
    }
}

fn exti0(mut ctxt: exti0::Context) {
    ctxt.async.a.post(&mut ctxt.threshold, ());
}

fn a(ctxt: a::Context) {
    asm::bkpt();
}
