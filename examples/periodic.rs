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
    fn init(cx: init::Context) {
        // omitted: initialization of `CYCCNT`

        cx.schedule.foo(Instant::now() + PERIOD.cycles()).unwrap();
    }

    #[task(schedule = [foo])]
    fn foo(cx: foo::Context) {
        let now = Instant::now();
        hprintln!("foo(scheduled = {:?}, now = {:?})", cx.scheduled, now).unwrap();

        cx.schedule.foo(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn UART0();
    }
};
