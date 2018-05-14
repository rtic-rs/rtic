#![feature(proc_macro)]
#![no_main]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_itm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error attempt to subtract with overflow
    device: stm32f103xx,

    tasks: {
        exti0: {
            interrupt: EXTI0,
            // ERROR priority must be in the range [1, 16]
            priority: 17,
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn idle(_ctxt: idle::Context) -> ! {
    loop {}
}

fn exti0(_ctxt: exti0::Context) {}
