#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Resource, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static STATE: bool = false;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [STATE],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [STATE],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn exti0(mut t: &mut Threshold, r: EXTI0::Resources) {
    // ERROR token should not outlive the critical section
    let t = r.STATE.claim(&mut t, |_state, t| t);
    //~^ error cannot infer an appropriate lifetime
}

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {}
