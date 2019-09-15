//! examples/capacity.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        rtfm::pend(Interrupt::UART0);
    }

    #[task(binds = UART0, spawn = [foo, bar])]
    fn uart0(c: uart0::Context) {
        c.spawn.foo(0).unwrap();
        c.spawn.foo(1).unwrap();
        c.spawn.foo(2).unwrap();
        c.spawn.foo(3).unwrap();

        c.spawn.bar().unwrap();
    }

    #[task(capacity = 4)]
    fn foo(_: foo::Context, x: u32) {
        hprintln!("foo({})", x).unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!("bar").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn UART1();
    }
};
