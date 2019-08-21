//! examples/preempt.rs

#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        rtfm::pend(Interrupt::UART0);
    }

    #[task(binds = UART0, priority = 1)]
    fn uart0(_: uart0::Context) {
        hprintln!("UART0 - start").unwrap();
        rtfm::pend(Interrupt::UART2);
        hprintln!("UART0 - end").unwrap();
        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = UART1, priority = 2)]
    fn uart1(_: uart1::Context) {
        hprintln!(" UART1").unwrap();
    }

    #[task(binds = UART2, priority = 2)]
    fn uart2(_: uart2::Context) {
        hprintln!(" UART2 - start").unwrap();
        rtfm::pend(Interrupt::UART1);
        hprintln!(" UART2 - end").unwrap();
    }
};
