//! examples/static.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use rtfm::app;

macro_rules! println {
    ($($tt:tt)*) => {
        if let Ok(mut stdout) = cortex_m_semihosting::hio::hstdout() {
            use core::fmt::Write;

            writeln!(stdout, $($tt)*).ok();
        }
    };
}

#[app(device = lm3s6965)]
const APP: () = {
    static KEY: u32 = ();

    #[init]
    fn init() {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);

        KEY = 0xdeadbeef;
    }

    #[interrupt(resources = [KEY])]
    fn UART0() {
        println!("UART0(KEY = {:#x})", resources.KEY);

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[interrupt(priority = 2, resources = [KEY])]
    fn UART1() {
        println!("UART1(KEY = {:#x})", resources.KEY);
    }
};
