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
    struct Resources {
        // A resource
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(_: init::Context) {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);

        // error: `shared` can't be accessed from this context
        // shared += 1;

        loop {}
    }

    // `shared` can be access from this context
    #[task(binds = UART0, resources = [shared])]
    fn uart0(c: uart0::Context) {
        *c.resources.shared += 1;

        hprintln!("UART0: shared = {}", c.resources.shared).unwrap();
    }

    // `shared` can be access from this context
    #[task(binds = UART1, resources = [shared])]
    fn uart1(c: uart1::Context) {
        *c.resources.shared += 1;

        hprintln!("UART1: shared = {}", c.resources.shared).unwrap();
    }
};
