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
        static STATE: bool = false;
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,
            // priority: 1,
            resources: [STATE],
        },

        exti1: {
            interrupt: EXTI1,
            priority: 2,
            resources: [STATE],
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn exti0(ctxt: exti0::Context) {
    // ERROR token should not outlive the critical section
    let op = &mut ctxt.priority;
    let p = ctxt.resources.STATE.claim(op, |_state, ip| ip);
    //~^ error cannot infer an appropriate lifetime
}

fn exti1(_ctxt: exti1::Context) {}
