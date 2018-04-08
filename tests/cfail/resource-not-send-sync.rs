#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

    resources: {
        static SHARED: bool = false;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [SHARED],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [SHARED],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn is_send<T>(_: &T) where T: Send {}
fn is_sync<T>(_: &T) where T: Sync {}

fn exti0(_t: &mut Threshold, r: EXTI0::Resources) {
    // ERROR resource proxies can't be shared between tasks
    is_sync(&r.SHARED);
    //~^ error `*const ()` cannot be shared between threads safely

    // ERROR resource proxies are not `Send`able across tasks
    is_send(&r.SHARED);
    //~^ error the trait bound `*const (): core::marker::Send` is not satisfied
}

fn exti1(_t: &mut Threshold, _r: EXTI1::Resources) {
}
