//! Two tasks running at the same priority with access to the same resource

#![deny(unsafe_code)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    // Resources that are plain data, not peripherals
    resources: {
        // Declaration of resources looks like the declaration of `static`
        // variables
        static COUNTER: u64 = 0;
    },

    tasks: {
        SYS_TICK: {
            path: sys_tick,
            priority: 1,
            // Both this task and TIM2 have access to the `COUNTER` resource
            resources: [COUNTER],
        },

        // An interrupt as a task
        TIM2: {
            // For interrupts the `enabled` field must be specified. It
            // indicates if the interrupt will be enabled or disabled once
            // `idle` starts
            enabled: true,
            path: tim2,
            priority: 1,
            resources: [COUNTER],
        },
    },
}

// when data resources are declared in the top `resources` field, `init` will
// have full access to them
fn init(_p: init::Peripherals, _r: init::Resources) {
    // ..
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

// As both tasks are running at the same priority one can't preempt the other.
// Thus both tasks have direct access to the resource
fn sys_tick(_t: &mut Threshold, r: SYS_TICK::Resources) {
    // ..

    **r.COUNTER += 1;

    // ..
}

fn tim2(_t: &mut Threshold, r: TIM2::Resources) {
    // ..

    **r.COUNTER += 1;

    // ..
}
