//! A showcase of the `app!` macro syntax
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static CO_OWNED: u32 = 0;
        static ON: bool = false;
        static OWNED: bool = false;
        static SHARED: bool = false;
    },

    init: {
        // This is the path to the `init` function
        //
        // `init` doesn't necessarily has to be in the root of the crate
        path: main::init,
    },

    idle: {
        // This is a path to the `idle` function
        //
        // `idle` doesn't necessarily has to be in the root of the crate
        path: main::idle,
        resources: [OWNED, SHARED],
    },

    tasks: {
        SYS_TICK: {
            path: sys_tick,
            // If omitted priority is assumed to be 1
            // priority: 1,
            resources: [CO_OWNED, ON, SHARED],
        },

        TIM2: {
            // Tasks are enabled, between `init` and `idle`, by default but they
            // can start disabled if `false` is specified here
            enabled: false,
            path: tim2,
            priority: 1,
            resources: [CO_OWNED],
        },
    },
}

mod main {
    use rtfm::{self, Resource, Threshold};

    pub fn init(_p: ::init::Peripherals, _r: ::init::Resources) {}

    pub fn idle(t: &mut Threshold, mut r: ::idle::Resources) -> ! {
        loop {
            *r.OWNED != *r.OWNED;

            if *r.OWNED {
                if r.SHARED.claim(t, |shared, _| *shared) {
                    rtfm::wfi();
                }
            } else {
                r.SHARED.claim_mut(t, |shared, _| *shared = !*shared);
            }
        }
    }
}

fn sys_tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    *r.ON = !*r.ON;

    *r.CO_OWNED += 1;
}

fn tim2(_t: &mut Threshold, mut r: TIM2::Resources) {
    *r.CO_OWNED += 1;
}
