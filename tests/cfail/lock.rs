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
        static MAX: u8 = 0;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [MAX, ON],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [ON],
        },

        EXTI2: {
            path: exti2,
            priority: 16,
            resources: [MAX],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn exti0(mut t: &mut Threshold, mut r: EXTI0::Resources) {
    // OK need to lock to access the resource
    if r.ON.claim(&mut t, |on, _| **on) {}

    // OK can claim a resource with maximum ceiling
    r.MAX.claim_mut(&mut t, |max, _| **max += 1);
}

fn exti1(mut t: &mut Threshold, r: EXTI1::Resources) {
    // ERROR no need to lock. Has direct access because priority == ceiling
    if (**r.ON).claim(&mut t, |on, _| **on) {
        //~^ error no method named `claim` found for type
    }

    if **r.ON {
        // OK
    }
}

fn exti2(_t: &mut Threshold, _r: EXTI2::Resources) {}
