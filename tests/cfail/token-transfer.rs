#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! { //~ error `*const ()` cannot be sent between threads safely
    device: stm32f103xx,

    resources: {
        static TOKEN: Option<Threshold> = None;
    },

    idle: {
        resources: [TOKEN],
    },

    tasks: {
        EXTI0: {
            path: exti0,
            resources: [TOKEN],
        },
    }
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle(_t: &mut Threshold, _r: idle::Resources) -> ! {
    loop {}
}

fn exti0(_t: &mut Threshold, _r: EXTI0::Resources) {}
