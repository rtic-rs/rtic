//! examples/periodic.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rtfm::cyccnt::{Instant, U32Ext};

const PERIOD: u32 = 8_000_000;

// NOTE: does NOT work on QEMU!
#[rtfm::app(device = lm3s6965, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    #[init(schedule = [foo])]
    fn init(c: init::Context) {
        c.schedule.foo(Instant::now() + PERIOD.cycles()).unwrap();
    }

    #[task(schedule = [foo])]
    fn foo(c: foo::Context) {
        let now = Instant::now();
        hprintln!("foo(scheduled = {:?}, now = {:?})", c.scheduled, now).unwrap();

        c.schedule.foo(c.scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn UART0();
    }
};
