//! examples/ramfunc.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        foo::spawn().unwrap();

        init::LateResources {}
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
    #[task(priority = 2)]
    fn bar(_: bar::Context) {
        foo::spawn().unwrap();
    }

    extern "C" {
        fn UART0();

        // run the task dispatcher from RAM
        #[link_section = ".data.UART1"]
        fn UART1();
    }
}
