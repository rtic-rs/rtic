#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error referenced constant has errors
    //~^ error could not evaluate constant
    //~| error constant evaluation error
    device: stm32f103xx,

    tasks: {
        SysTick: {
            path: sys_tick,
            // ERROR priority must be in the range [1, 16]
            priority: 17,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}

fn sys_tick() {}
