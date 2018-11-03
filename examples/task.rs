//! examples/task.rs

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
        spawn.foo().unwrap();
    }

    #[task(spawn = [bar, baz])]
    fn foo() {
        println!("foo");

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
        println!("bar");

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(priority = 2)]
    fn baz() {
        println!("baz");
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn UART0();
        fn UART1();
    }
};
