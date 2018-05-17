#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
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
        static ON: bool = false;
        static MAX: u8 = 0;
        static OWNED: bool = false;
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,
            // priority: 1,
            resources: [MAX, ON],
        },

        exti1: {
            interrupt: EXTI1,
            priority: 2,
            resources: [ON, OWNED],
        },

        exti2: {
            interrupt: EXTI2,
            priority: 16,
            resources: [MAX],
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

#[allow(non_snake_case)]
fn exti0(mut ctxt: exti0::Context) {
    let exti0::Resources { ON, mut MAX } = ctxt.resources;
    let p = &mut ctxt.priority;

    // ERROR need to lock to access the resource because priority < ceiling
    {
        let _on = ON.borrow(p);
        //~^ error type mismatch resolving
    }

    // OK need to lock to access the resource
    if ON.claim(p, |on, _| *on) {}

    // OK can claim a resource with maximum ceiling
    MAX.claim_mut(p, |max, _| *max += 1);
}

#[allow(non_snake_case)]
fn exti1(ctxt: exti1::Context) {
    let exti1::Resources { OWNED, .. } = ctxt.resources;

    // OK to directly access the resource because this task is the only owner
    if *OWNED {}
}

fn exti2(_ctxt: exti2::Context) {}
