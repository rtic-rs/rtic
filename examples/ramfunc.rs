//! examples/ramfunc.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [bar])]
    fn init(c: init::Context) {
        c.spawn.bar().unwrap();
    }

    #[inline(never)]
    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    // run this task from RAM
    #[inline(never)]
    #[link_section = ".data.bar"]
    #[task(priority = 2, spawn = [foo])]
    fn bar(c: bar::Context) {
        c.spawn.foo().unwrap();
    }

    extern "C" {
        fn UART0();

        // run the task dispatcher from RAM
        #[link_section = ".data.UART1"]
        fn UART1();
    }
};
