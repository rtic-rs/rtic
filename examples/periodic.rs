//! examples/periodic.rs

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

const PERIOD: u32 = 8_000_000;

// NOTE: does NOT work on QEMU!
#[app(device = lm3s6965)]
const APP: () = {
    #[init(schedule = [foo])]
    fn init() {
        schedule.foo(Instant::now() + PERIOD.cycles()).unwrap();
    }

    #[task(schedule = [foo])]
    fn foo() {
        let now = Instant::now();
        println!("foo(scheduled = {:?}, now = {:?})", scheduled, now);

        schedule.foo(scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn UART0();
    }
};
