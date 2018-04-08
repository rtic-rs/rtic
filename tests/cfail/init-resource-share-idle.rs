#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ proc macro panicked
    device: stm32f103xx,

    resources: {
        static BUFFER: [u8; 16] = [0; 16];
    },

    init: {
        resources: [BUFFER],
    },

    idle: {
        resources: [BUFFER],
        //~^ error: this resource is owned by `init` and can't be shared
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle(_r: init::Resources) -> ! {
    loop {}
}
