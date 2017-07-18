// error-pattern: the name `EXTI0` is defined multiple times

#![deny(warnings)]
#![feature(proc_macro)]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}

task!(EXTI0, exti0);

fn exti0(_t: Threshold, _r: EXTI0::Resources) {}

task!(EXTI0, exti1);

fn exti1(_t: Threshold, _r: EXTI0::Resources) {}
