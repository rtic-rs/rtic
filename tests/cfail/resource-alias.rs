#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error proc macro panicked
    device: stm32f103xx,

    resources: {
        // resource `MAX` listed twice
        MAX: u8 = 0;
        MAX: u16 = 0;
    },

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}
