//! Nesting claims and how the preemption threshold works
//!
//! If you run this program you'll hit the breakpoints as indicated by the
//! letters in the comments: A, then B, then C, etc.
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Resource, Threshold};
use stm32f103xx::Interrupt;

app! {
    device: stm32f103xx,

    resources: {
        static LOW: u64 = 0;
        static HIGH: u64 = 0;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [LOW, HIGH],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [LOW],
        },

        EXTI2: {
            path: exti2,
            priority: 3,
            resources: [HIGH],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    // A
    rtfm::bkpt();

    // Sets task `exti0` as pending
    //
    // Because `exti0` has higher priority than `idle` it will be executed
    // immediately
    rtfm::set_pending(Interrupt::EXTI0); // ~> exti0

    loop {
        rtfm::wfi();
    }
}

#[allow(non_snake_case)]
fn exti0(t: &mut Threshold, EXTI0::Resources { mut LOW, mut HIGH }: EXTI0::Resources) {
    // Because this task has a priority of 1 the preemption threshold `t` also
    // starts at 1

    // B
    rtfm::bkpt();

    // Because `exti1` has higher priority than `exti0` it can preempt it
    rtfm::set_pending(Interrupt::EXTI1); // ~> exti1

    // A claim creates a critical section
    LOW.claim_mut(t, |_low, t| {
        // This claim increases the preemption threshold to 2
        //
        // 2 is just high enough to not race with task `exti1` for access to the
        // `LOW` resource

        // D
        rtfm::bkpt();

        // Now `exti1` can't preempt this task because its priority is equal to
        // the current preemption threshold
        rtfm::set_pending(Interrupt::EXTI1);

        // But `exti2` can, because its priority is higher than the current
        // preemption threshold
        rtfm::set_pending(Interrupt::EXTI2); // ~> exti2

        // F
        rtfm::bkpt();

        // Claims can be nested
        HIGH.claim_mut(t, |_high, _| {
            // This claim increases the preemption threshold to 3

            // Now `exti2` can't preempt this task
            rtfm::set_pending(Interrupt::EXTI2);

            // G
            rtfm::bkpt();
        });

        // Upon leaving the critical section the preemption threshold drops back
        // to 2 and `exti2` immediately preempts this task
        // ~> exti2
    });

    // Once again the preemption threshold drops but this time to 1. Now the
    // pending `exti1` task can preempt this task
    // ~> exti1
}

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {
    // C, I
    rtfm::bkpt();
}

fn exti2(_t: &mut Threshold, _r: EXTI2::Resources) {
    // E, H
    rtfm::bkpt();
}
