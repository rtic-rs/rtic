//! examples/schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::hprintln;
use rtfm::Instant;

// NOTE: does NOT work on QEMU!
#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init(schedule = [foo, bar])]
    fn init(c: init::Context) {
        let now = Instant::now();

        hprintln!("init @ {:?}", now).unwrap();

        // Schedule `foo` to run 8e6 cycles (clock cycles) in the future
        c.schedule.foo(now + 8_000_000.cycles()).unwrap();

        // Schedule `bar` to run 4e6 cycles in the future
        c.schedule.bar(now + 4_000_000.cycles()).unwrap();
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo  @ {:?}", Instant::now()).unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!("bar  @ {:?}", Instant::now()).unwrap();
    }

    extern "C" {
        fn UART0();
    }
};
