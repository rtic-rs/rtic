#![deny(warnings)]
#![feature(proc_macro)]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! {
    //~^ error no associated item named `SYS_TICK` found for type
    //~| error no associated item named `SYS_TICK` found for type
    device: stm32f103xx,

    tasks: {
        // ERROR exceptions can't be enabled / disabled here
        SYS_TICK: {
            enabled: true,
            priority: 1,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}
