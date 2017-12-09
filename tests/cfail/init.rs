#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::app;

app! { //~ error mismatched types
    device: stm32f103xx,
}

// ERROR `init` must have signature `fn (init::Peripherals)`
fn init() {}

fn idle() -> ! {
    loop {}
}
