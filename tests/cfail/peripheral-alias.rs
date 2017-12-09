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
        EXTI0: {
            enabled: true,
            priority: 1,
            // ERROR peripheral appears twice in this list
            resources: [GPIOA, GPIOA],
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}
