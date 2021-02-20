//! examples/periodic.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

// NOTE: does NOT work on QEMU!
#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::hprintln;
    use rtic::cyccnt::{Instant, U32Ext};

    const PERIOD: u32 = 8_000_000;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        // omitted: initialization of `CYCCNT`

        foo::schedule(cx.start + PERIOD.cycles()).unwrap();

        (init::LateResources {}, init::Monotonics())
    }

    #[task]
    fn foo(cx: foo::Context) {
        let now = Instant::now();
        hprintln!("foo(scheduled = {:?}, now = {:?})", cx.scheduled, now).unwrap();

        foo::schedule(cx.scheduled + PERIOD.cycles()).unwrap();
    }
}
