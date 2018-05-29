#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![feature(proc_macro_gen)]
#![no_main]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_itm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error proc macro panicked
    device: stm32f103xx,

    resources: {
        static MAX: u8 = 0;
        static MAX: u16 = 0; //~ error this resource name appears more than once in this list
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,
            resources: [
                MAX,
                MAX, //~ error this resource name appears more than once in this list
            ],
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {}
