#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static ON: bool = false;
    },

    tasks: {
        EXTI0: {
            enabled: true,
            path: exti0,
            priority: 1,
            resources: [ON],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn exti0(_r: EXTI0::Resources) {}

// ERROR can't override the task handler specified in `app!`
task!(EXTI0, exti1);
//~^ error cannot find value `EXTI0`

fn exti1(_t: &mut Threshold, _r: EXTI0::Resources) {}
