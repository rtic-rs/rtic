//! examples/static.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    extern "C" {
        static KEY: u32;
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);

        init::LateResources { KEY: 0xdeadbeef }
    }

    #[task(binds = UART0, resources = [KEY])]
    fn uart0(c: uart0::Context) {
        hprintln!("UART0(KEY = {:#x})", c.resources.KEY).unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = UART1, priority = 2, resources = [KEY])]
    fn uart1(c: uart1::Context) {
        hprintln!("UART1(KEY = {:#x})", c.resources.KEY).unwrap();
    }
};
