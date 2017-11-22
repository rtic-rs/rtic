#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Resource, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static A: u8 = 0;
        static B: u8 = 0;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [A, B],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [A, B],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn exti0(mut ot: &mut Threshold, r: EXTI0::Resources) {
    r.A.claim(&mut ot, |_a, mut _it| {
        //~^ error cannot borrow `ot` as mutable more than once at a time
        // ERROR must use inner token `it` instead of the outer one (`ot`)
        r.B.claim(&mut ot, |_b, _| {})
    });
}

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {}
