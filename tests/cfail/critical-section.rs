#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Resource, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static ON: bool = false;
    },

    idle: {
        resources: [ON],
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [ON],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    let state = rtfm::atomic(t, |t| {
        // ERROR borrow can't escape this *global* critical section
        r.ON.borrow(t) //~ error cannot infer an appropriate lifetime
    });

    let state = r.ON.claim(t, |state, _t| {
        // ERROR borrow can't escape this critical section
        state //~ error cannot infer an appropriate lifetime
    });

    loop {}
}

fn exti0(_t: &mut Threshold, _r: EXTI0::Resources) {}
