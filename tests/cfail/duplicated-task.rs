#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error proc macro panicked
    device: stm32f103xx,

    tasks: {
        a: {
            interrupt: EXTI0, //~ error this interrupt is already bound to another task
            priority: 1,
        },

        b: {
            interrupt: EXTI0,
            priority: 2,
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {}

fn idle(_ctxt: idle::Context) -> ! {}
