//! examples/task.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};
use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [foo])]
    fn init() {
        spawn.foo().unwrap();
    }

    #[task(spawn = [bar, baz])]
    fn foo() {
        hprintln!("foo").unwrap();

        // spawns `bar` onto the task scheduler
        // `foo` and `bar` have the same priority so `bar` will not run until
        // after `foo` terminates
        spawn.bar().unwrap();

        // spawns `baz` onto the task scheduler
        // `baz` has higher priority than `foo` so it immediately preempts `foo`
        spawn.baz().unwrap();
    }

    #[task]
    fn bar() {
        hprintln!("bar").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(priority = 2)]
    fn baz() {
        hprintln!("baz").unwrap();
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn UART0();
        fn UART1();
    }
};
