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
    struct Resources {
        key: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);

        init::LateResources { key: 0xdeadbeef }
    }

    #[task(binds = UART0, resources = [&key])]
    fn uart0(c: uart0::Context) {
        hprintln!("UART0(key = {:#x})", c.resources.key).unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = UART1, priority = 2, resources = [&key])]
    fn uart1(c: uart1::Context) {
        hprintln!("UART1(key = {:#x})", c.resources.key).unwrap();
    }
};
