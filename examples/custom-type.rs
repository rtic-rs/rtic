#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

pub struct Foo;

app! {
    device: stm32f103xx,

    resources: {
        static CO_OWNED: Foo = Foo;
        static ON: Foo = Foo;
        static OWNED: Foo = Foo;
        static SHARED: Foo = Foo;
    },

    idle: {
        resources: [OWNED, SHARED],
    },

    tasks: {
        SYS_TICK: {
            path: sys_tick,
            resources: [CO_OWNED, ON, SHARED],
        },

        TIM2: {
            enabled: false,
            path: tim2,
            priority: 1,
            resources: [CO_OWNED],
        },
    },
}

fn init(_p: ::init::Peripherals, _r: ::init::Resources) {}

fn idle(_t: &mut Threshold, _r: ::idle::Resources) -> ! {
    loop {}
}

fn sys_tick(_t: &mut Threshold, _r: SYS_TICK::Resources) {}

fn tim2(_t: &mut Threshold, _r: TIM2::Resources) {}
