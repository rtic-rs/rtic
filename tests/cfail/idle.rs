#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ mismatched types
    device: stm32f103xx,
}

fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

// ERROR `idle` must be a diverging function
fn idle(_ctxt: idle::Context) {}
