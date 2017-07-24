#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error no associated item named `EXTI0` found for type
    device: stm32f103xx,

    tasks: {
        // ERROR `enabled` needs to be specified for interrupts
        EXTI0: {
            priority: 1,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}
