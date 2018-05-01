#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error proc macro panicked
    device: stm32f103xx, //~ no variant named `SYS_TICK` found for type `stm32f103xx::Interrupt`

    tasks: {
        sys_tick: {
            interrupt: SYS_TICK, // ERROR can't bind to exception
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn idle(_ctxt: idle::Context) -> ! {
    loop {}
}

fn sys_tick(_ctxt: sys_tick::Context) {}
