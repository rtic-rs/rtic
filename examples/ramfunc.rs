//! examples/ramfunc.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};
use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [bar])]
    fn init() {
        spawn.bar().unwrap();
    }

    #[inline(never)]
    #[task]
    fn foo() {
        hprintln!("foo").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    // run this task from RAM
    #[inline(never)]
    #[link_section = ".data.bar"]
    #[task(priority = 2, spawn = [foo])]
    fn bar() {
        spawn.foo().unwrap();
    }

    extern "C" {
        fn UART0();

        // run the task dispatcher from RAM
        #[link_section = ".data.UART1"]
        fn UART1();
    }
};
