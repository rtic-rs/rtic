#![allow(warnings)]
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

    tasks: {
        exti0: {
            interrupt: EXTI0,
        },
    },
}

#[inline(always)]
fn init(mut _ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

#[inline(always)]
fn idle(_ctxt: idle::Context) -> ! {
    loop {
        asm::wfi();
    }
}

fn exti0(_ctxt: exti0::Context) {}
