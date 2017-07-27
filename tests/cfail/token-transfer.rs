#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! { //~ error bound `rtfm::Threshold: core::marker::Send` is not satisfied
    device: stm32f103xx,

    resources: {
        static TOKEN: Option<Threshold> = None;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [TOKEN],
        },
    }
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn exti0(_t: &mut Threshold, _r: EXTI0::Resources) {}
