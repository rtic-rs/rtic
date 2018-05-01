#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static SHARED: bool = false;
    },

    tasks: {
        exti0: {
            interrupt: EXTI0,
            // priority: 1,
            resources: [SHARED],
        },

        exti1: {
            interrupt: EXTI1,
            priority: 2,
            resources: [SHARED],
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn idle(_ctxt: idle::Context) -> ! {
    loop {}
}

fn is_send<T>(_: &T)
where
    T: Send,
{
}
fn is_sync<T>(_: &T)
where
    T: Sync,
{
}

fn exti0(ctxt: exti0::Context) {
    // ERROR resource proxies can't be shared between tasks
    is_sync(&ctxt.resources.SHARED);
    //~^ error `*const ()` cannot be shared between threads safely

    // ERROR resource proxies are not `Send`able across tasks
    is_send(&ctxt.resources.SHARED);
    //~^ error the trait bound `*const (): core::marker::Send` is not satisfied
}

fn exti1(_ctxt: exti1::Context) {}
