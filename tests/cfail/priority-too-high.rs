#![deny(warnings)]
#![feature(proc_macro)]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! { //~ error attempt to subtract with overflow
    device: stm32f103xx,

    tasks: {
        SYS_TICK: {
            // ERROR priority must be in the range [1, 16]
            priority: 17,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}

task!(SYS_TICK, sys_tick);

fn sys_tick(_: Threshold, _: SYS_TICK::Resources) {}
