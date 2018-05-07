#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_itm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error proc macro panicked
    device: stm32f103xx,

    tasks: {
        exti0: {
            interrupt: EXTI0,
            priority: 0, //~ error this value is outside the valid range of `(1, 255)`
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {}

fn idle(_ctxt: idle::Context) -> ! {
    loop {}
}

fn exti0(_ctxt: exti0::Context) {}
