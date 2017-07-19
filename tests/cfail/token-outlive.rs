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
        STATE: bool = false;
    },

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
            resources: [STATE],
        },

        EXTI1: {
            enabled: true,
            priority: 2,
            resources: [STATE],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

task!(EXTI0, exti0);

fn exti0(mut t: &mut Threshold, r: EXTI0::Resources) {
    // ERROR token should not outlive the critical section
    let t = r.STATE.claim(&mut t, |_state, t| t);
    //~^ error cannot infer an appropriate lifetime
}

task!(EXTI1, exti1);

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {}
