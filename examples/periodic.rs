//! examples/periodic.rs

#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rtic::time::{self, prelude::*, units::*, Instant};

const PERIOD: Milliseconds<i32> = Milliseconds(125);

// NOTE: does NOT work on QEMU!
#[rtic::app(device = lm3s6965, monotonic = crate::CYCCNT, sys_timer_freq = 64_000_000)]
const APP: () = {
    #[init(schedule = [foo])]
    fn init(cx: init::Context) {
        // omitted: initialization of `CYCCNT`

        cx.schedule.foo(CYCCNT::now() + PERIOD).unwrap();
    }

    #[task(schedule = [foo])]
    fn foo(cx: foo::Context) {
        let now = CYCCNT::now();
        hprintln!("foo(scheduled = {:?}, now = {:?})", cx.scheduled, now).unwrap();

        cx.schedule.foo(cx.scheduled + PERIOD).unwrap();
    }

    extern "C" {
        fn UART0();
    }
};

/// Implementation of the `Monotonic` trait based on CYCle CouNTer
#[derive(Debug)]
pub struct CYCCNT;

impl rtic::Monotonic for CYCCNT {
    unsafe fn reset() {
        (0xE0001004 as *mut u32).write_volatile(0)
    }
}

impl time::Clock for CYCCNT {
    type Rep = i32;

    // the period of 64 MHz
    const PERIOD: time::Period = time::Period::new(1, 64_000_000);

    fn now() -> Instant<Self> {
        let ticks = DWT::get_cycle_count();

        Instant::new(ticks as i32)
    }
}
