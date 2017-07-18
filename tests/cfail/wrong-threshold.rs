#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        A: u8 = 0;
        B: u8 = 0;
    },

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
            resources: [A, B],
        },

        EXTI1: {
            enabled: true,
            priority: 2,
            resources: [A, B],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

task!(EXTI0, exti0);

fn exti0(mut ot: Threshold, r: EXTI0::Resources) {
    r.A.claim(&mut ot, |_a, mut _it| {
        //~^ error cannot borrow `ot` as mutable more than once at a time
        //~| error cannot borrow `ot` as mutable more than once at a time
        // ERROR must use inner token `it` instead of the outer one (`ot`)
        r.B.claim(&mut ot, |_b, _| {})
    });
}

task!(EXTI1, exti1);

fn exti1(_t: Threshold, r: EXTI1::Resources) {}
