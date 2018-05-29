#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![feature(proc_macro_gen)]
#![no_main]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_itm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error no variant named `EXTI33` found for type `stm32f103xx::Interrupt`
    device: stm32f103xx,

    tasks: {
        exti33: {
            interrupt: EXTI33,
        },
    },
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn exti33(_ctxt: exti33::Context) {}
