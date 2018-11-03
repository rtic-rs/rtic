//! examples/baseline.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use rtfm::app;

macro_rules! println {
    ($($tt:tt)*) => {
        if let Ok(mut stdout) = cortex_m_semihosting::hio::hstdout() {
            use core::fmt::Write;

            writeln!(stdout, $($tt)*).ok();
        }
    };
}

// NOTE: does NOT properly work on QEMU
#[app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [foo])]
    fn init() {
        println!("init(baseline = {:?})", start);

        // `foo` inherits the baseline of `init`: `Instant(0)`
        spawn.foo().unwrap();
    }

    #[task(schedule = [foo])]
    fn foo() {
        static mut ONCE: bool = true;

        println!("foo(baseline = {:?})", scheduled);

        if *ONCE {
            *ONCE = false;

            rtfm::pend(Interrupt::UART0);
        } else {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[interrupt(spawn = [foo])]
    fn UART0() {
        println!("UART0(baseline = {:?})", start);

        // `foo` inherits the baseline of `UART0`: its `start` time
        spawn.foo().unwrap();
    }

    extern "C" {
        fn UART1();
    }
};
