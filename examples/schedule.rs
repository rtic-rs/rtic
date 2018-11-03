//! examples/schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use rtfm::{app, Instant};

macro_rules! println {
    ($($tt:tt)*) => {
        if let Ok(mut stdout) = cortex_m_semihosting::hio::hstdout() {
            use core::fmt::Write;

            writeln!(stdout, $($tt)*).ok();
        }
    };
}

// NOTE: does NOT work on QEMU!
#[app(device = lm3s6965)]
const APP: () = {
    #[init(schedule = [foo, bar])]
    fn init() {
        let now = Instant::now();

        println!("init @ {:?}", now);

        // Schedule `foo` to run 8e6 cycles (clock cycles) in the future
        schedule.foo(now + 8_000_000.cycles()).unwrap();

        // Schedule `bar` to run 4e6 cycles in the future
        schedule.bar(now + 4_000_000.cycles()).unwrap();
    }

    #[task]
    fn foo() {
        println!("foo  @ {:?}", Instant::now());
    }

    #[task]
    fn bar() {
        println!("bar  @ {:?}", Instant::now());
    }

    extern "C" {
        fn UART0();
    }
};
