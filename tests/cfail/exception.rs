#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error proc macro panicked
    device: stm32f103xx,

    tasks: {
        // ERROR exceptions can't be enabled / disabled here
        SysTick: {
            enabled: true,
            priority: 1,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}
