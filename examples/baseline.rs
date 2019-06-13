//! examples/baseline.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

// NOTE: does NOT properly work on QEMU
#[rtfm::app(device = lm3s6965, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    #[init(spawn = [foo])]
    fn init(c: init::Context) {
        hprintln!("init(baseline = {:?})", c.start).unwrap();

        // `foo` inherits the baseline of `init`: `Instant(0)`
        c.spawn.foo().unwrap();
    }

    #[task(schedule = [foo])]
    fn foo(c: foo::Context) {
        static mut ONCE: bool = true;

        hprintln!("foo(baseline = {:?})", c.scheduled).unwrap();

        if *ONCE {
            *ONCE = false;

            rtfm::pend(Interrupt::UART0);
        } else {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[interrupt(spawn = [foo])]
    fn UART0(c: UART0::Context) {
        hprintln!("UART0(baseline = {:?})", c.start).unwrap();

        // `foo` inherits the baseline of `UART0`: its `start` time
        c.spawn.foo().unwrap();
    }

    extern "C" {
        fn UART1();
    }
};
