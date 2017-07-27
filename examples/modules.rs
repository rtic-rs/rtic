//! Using paths and modules
#![deny(unsafe_code)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static CO_OWNED: u32 = 0;
        static ON: bool = false;
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
            path: tasks::sys_tick,
            priority: 1,
            resources: [CO_OWNED, ON, SHARED],
        },

        TIM2: {
            enabled: true,
            path: tasks::tim2,
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

    pub fn sys_tick(_t: &mut Threshold, r: ::SYS_TICK::Resources) {
        **r.ON = !**r.ON;

        **r.CO_OWNED += 1;
    }

    pub fn tim2(_t: &mut Threshold, r: ::TIM2::Resources) {
        **r.CO_OWNED += 1;
    }
}
