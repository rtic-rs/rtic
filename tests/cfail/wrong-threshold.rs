#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_itm;
extern crate stm32f103xx;

use rtfm::{app, Resource};

app! {
    device: stm32f103xx,

    resources: {
        static A: u8 = 0;
        static B: u8 = 0;
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,
            // priority: 1,
            resources: [A, B],
        },

        exti1: {
            interrupt: EXTI1,
            priority: 2,
            resources: [A, B],
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn idle(_ctxt: idle::Context) -> ! {
    loop {}
}

fn exti0(mut ctxt: exti0::Context) {
    let ot = &mut ctxt.threshold;
    let exti0::Resources { A, B } = ctxt.resources;

    A.claim(ot, |_a, _it| {
        //~^ error closure requires unique access to `ot` but `*ot` is already borrowed
        // ERROR must use inner token `it` instead of the outer one (`ot`)
        B.claim(ot, |_b, _| {})
    });
}

fn exti1(_ctxt: exti1::Context) {}
