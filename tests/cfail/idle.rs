// error-pattern: mismatched types
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

app! {
    device: stm32f103xx,

    idle: {},
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

// ERROR `idle` must be a diverging function
fn idle(_ctxt: idle::Context) {}
