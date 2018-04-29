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

    tasks: {
        exti0: {
            interrupt: EXTI0,
        },
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

fn exti0(ctxt: exti0::Context) {}
