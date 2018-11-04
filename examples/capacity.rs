//! examples/capacity.rs

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
    #[init]
    fn init() {
        rtfm::pend(Interrupt::UART0);
    }

    #[interrupt(spawn = [foo, bar])]
    fn UART0() {
        spawn.foo(0).unwrap();
        spawn.foo(1).unwrap();
        spawn.foo(2).unwrap();
        spawn.foo(3).unwrap();

        spawn.bar().unwrap();
    }

    #[task(capacity = 4)]
    fn foo(x: u32) {
        hprintln!("foo({})", x).unwrap();
    }

    #[task]
    fn bar() {
        hprintln!("bar").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn UART1();
    }
};
