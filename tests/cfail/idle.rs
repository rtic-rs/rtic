#![deny(warnings)]
#![feature(proc_macro)]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error mismatched types
    device: stm32f103xx,
}

fn init(_p: init::Peripherals) {}

// ERROR `idle` must be a diverging function
fn idle() {}
