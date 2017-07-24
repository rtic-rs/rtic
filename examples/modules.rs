//! Using paths and modules

#![deny(unsafe_code)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static CO_OWNED: u32 = 0;
        static OWNED: bool = false;
        static SHARED: bool = false;
    },

    init: {
        path: main::init,
    },

    idle: {
        path: main::idle,
        resources: [OWNED, SHARED],
    },

    tasks: {
        SYS_TICK: {
            priority: 1,
            resources: [CO_OWNED, SHARED],
        },

        TIM2: {
            enabled: true,
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
                if r.SHARED.claim(t, |shared, _| **shared) {
                    rtfm::wfi();
                }
            } else {
                r.SHARED.claim_mut(t, |shared, _| **shared = !**shared);
            }
        }
    }
}

pub mod tasks {
    use rtfm::Threshold;

    task!(SYS_TICK, sys_tick, Locals {
        static STATE: bool = true;
    });

    fn sys_tick(_t: &mut Threshold, l: &mut Locals, r: ::SYS_TICK::Resources) {
        *l.STATE = !*l.STATE;

        **r.CO_OWNED += 1;
    }

    task!(TIM2, tim2);

    fn tim2(_t: &mut Threshold, r: ::TIM2::Resources) {
        **r.CO_OWNED += 1;
    }
}
