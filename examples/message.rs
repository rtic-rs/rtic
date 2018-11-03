//! examples/message.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
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
    #[init(spawn = [foo])]
    fn init() {
        spawn.foo(/* no message */).unwrap();
    }

    #[task(spawn = [bar])]
    fn foo() {
        static mut COUNT: u32 = 0;

        println!("foo");

        spawn.bar(*COUNT).unwrap();
        *COUNT += 1;
    }

    #[task(spawn = [baz])]
    fn bar(x: u32) {
        println!("bar({})", x);

        spawn.baz(x + 1, x + 2).unwrap();
    }

    #[task(spawn = [foo])]
    fn baz(x: u32, y: u32) {
        println!("baz({}, {})", x, y);

        if x + y > 4 {
            debug::exit(debug::EXIT_SUCCESS);
        }

        spawn.foo().unwrap();
    }

    extern "C" {
        fn UART0();
    }
};
