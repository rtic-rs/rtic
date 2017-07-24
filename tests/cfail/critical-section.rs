#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Resource, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static STATE: bool = false;
    },

    idle: {
        resources: [STATE],
    },

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
            resources: [STATE],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    let state = rtfm::atomic(|cs| {
        // ERROR borrow can't escape this *global* critical section
        r.STATE.borrow(cs) //~ error cannot infer an appropriate lifetime
    });

    let state = r.STATE.claim(t, |state, _t| {
        // ERROR borrow can't escape this critical section
        state //~ error cannot infer an appropriate lifetime
    });

    loop {}
}

task!(EXTI0, exti0);

fn exti0(_t: &mut Threshold, _r: EXTI0::Resources) {}
