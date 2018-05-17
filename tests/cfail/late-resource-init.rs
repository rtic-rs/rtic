#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_main]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_itm;
extern crate stm32f103xx;

use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static A: u8 = 0;
        static LATE: u8;
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,
            // priority: 1,
            resources: [A, LATE],
        },

        exti1: {
            interrupt: EXTI1,
            priority: 2,
            resources: [A, LATE],
        },
    },
}

fn init(ctxt: init::Context) -> init::LateResources {
    // Tried to use a resource that's not yet initialized:
    let _late = ctxt.resources.LATE;
    //~^ error: no field `LATE` on type `init::Resources`

    init::LateResources { LATE: 0 }
}

fn exti0(_ctxt: exti0::Context) {}

fn exti1(_ctxt: exti1::Context) {}
