#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! { //~ error bound `rtfm::Threshold: std::marker::Send` is not satisfied
    device: stm32f103xx,

    resources: {
        TOKEN: Option<Threshold> = None;
    },

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
            resources: [TOKEN],
        },
    }
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

task!(EXTI0, exti0);

fn exti0(_t: &mut Threshold, _r: EXTI0::Resources) {}
