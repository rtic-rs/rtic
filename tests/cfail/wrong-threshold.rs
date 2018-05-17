#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_main]
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

fn exti0(mut ctxt: exti0::Context) {
    let op = &mut ctxt.priority;
    let exti0::Resources { A, B } = ctxt.resources;

    A.claim(op, |_a, _ip| {
        //~^ error closure requires unique access to `op` but `*op` is already borrowed
        // ERROR must use inner token `_ip` instead of the outer one (`op`)
        B.claim(op, |_b, _| {})
    });
}

fn exti1(_ctxt: exti1::Context) {}
