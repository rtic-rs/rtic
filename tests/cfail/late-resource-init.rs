#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static A: u8 = 0;
        static LATE: u8;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [A, LATE],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [A, LATE],
        },
    },
}

fn init(_p: init::Peripherals, r: init::Resources) -> init::LateResources {
    // Try to use a resource that's not yet initialized:
    r.LATE;
    //~^ error: no field `LATE`

    init::LateResources {
        LATE: 0,
    }
}

fn idle() -> ! {
    loop {}
}

fn exti0(_t: &mut Threshold, _r: EXTI0::Resources) {}

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {}
