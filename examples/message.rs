//! examples/message.rs

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
        spawn.foo(/* no message */).unwrap();
    }

    #[task(spawn = [bar])]
    fn foo() {
        static mut COUNT: u32 = 0;

        hprintln!("foo").unwrap();

        spawn.bar(*COUNT).unwrap();
        *COUNT += 1;
    }

    #[task(spawn = [baz])]
    fn bar(x: u32) {
        hprintln!("bar({})", x).unwrap();

        spawn.baz(x + 1, x + 2).unwrap();
    }

    #[task(spawn = [foo])]
    fn baz(x: u32, y: u32) {
        hprintln!("baz({}, {})", x, y).unwrap();

        if x + y > 4 {
            debug::exit(debug::EXIT_SUCCESS);
        }

        spawn.foo().unwrap();
    }

    extern "C" {
        fn UART0();
    }
};
