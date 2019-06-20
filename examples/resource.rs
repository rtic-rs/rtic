//! examples/resource.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    // A resource
    static mut SHARED: u32 = 0;

    #[init]
    fn init(_: init::Context) {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);

        // error: `SHARED` can't be accessed from this context
        // SHARED += 1;

        loop {}
    }

    // `SHARED` can be access from this context
    #[task(binds = UART0, resources = [SHARED])]
    fn uart0(c: uart0::Context) {
        *c.resources.SHARED += 1;

        hprintln!("UART0: SHARED = {}", c.resources.SHARED).unwrap();
    }

    // `SHARED` can be access from this context
    #[task(binds = UART1, resources = [SHARED])]
    fn uart1(c: uart1::Context) {
        *c.resources.SHARED += 1;

        hprintln!("UART1: SHARED = {}", c.resources.SHARED).unwrap();
    }
};
