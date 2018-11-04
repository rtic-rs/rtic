//! examples/resource.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    // A resource
    static mut SHARED: u32 = 0;

    #[init]
    fn init() {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);
    }

    #[idle]
    fn idle() -> ! {
        debug::exit(debug::EXIT_SUCCESS);

        // error: `SHARED` can't be accessed from this context
        // SHARED += 1;

        loop {}
    }

    // `SHARED` can be access from this context
    #[interrupt(resources = [SHARED])]
    fn UART0() {
        *resources.SHARED += 1;

        hprintln!("UART0: SHARED = {}", resources.SHARED).unwrap();
    }

    // `SHARED` can be access from this context
    #[interrupt(resources = [SHARED])]
    fn UART1() {
        *resources.SHARED += 1;

        hprintln!("UART1: SHARED = {}", resources.SHARED).unwrap();
    }
};
