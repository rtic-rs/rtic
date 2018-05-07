#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_itm;
extern crate stm32f103xx;

use rtfm::{app, Resource};

app! {
    device: stm32f103xx,

    resources: {
        static ON: bool = false;
    },

    idle: {
        resources: [ON],
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,
            resources: [ON],
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn idle(mut ctxt: idle::Context) -> ! {
    let t = &mut ctxt.threshold;
    let on = ctxt.resources.ON;

    let state = rtfm::atomic(t, |t| {
        // ERROR borrow can't escape this *global* critical section
        on.borrow(t) //~ error cannot infer an appropriate lifetime
    });

    let state = on.claim(t, |state, _t| {
        // ERROR borrow can't escape this critical section
        state //~ error cannot infer an appropriate lifetime
    });

    loop {}
}

fn exti0(_ctxt: exti0::Context) {}
