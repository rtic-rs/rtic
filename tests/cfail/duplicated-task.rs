#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! {
    //~^ error proc macro panicked
    device: stm32f103xx,

    tasks: {
        SysTick: {
            priority: 1,
        },

        SysTick: {
            priority: 2,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {}
