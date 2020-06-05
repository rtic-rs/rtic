//! examples/schedule.rs

#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use panic_halt as _;
use cortex_m::peripheral::DWT;
use rtic::time::{self, instant::Instant, prelude::*, time_units::*};

// NOTE: does NOT work on QEMU!
#[rtic::app(device = lm3s6965, monotonic = crate::CYCCNT, sys_timer_freq = 64_000_000)]
const APP: () = {
    #[init(schedule = [foo, bar])]
    fn init(mut cx: init::Context) {
        // Initialize (enable) the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = cx.start; // the start time of the system

        hprintln!(
            "init @ {:?}",
            CYCCNT::now().elapsed_since_epoch::<Microseconds<i32>>()
        )
        .unwrap();

        // Schedule `foo` to run 1 second in the future
        cx.schedule.foo(now + 1.seconds()).unwrap();

        // Schedule `bar` to run 2 seconds in the future
        cx.schedule.bar(now + 2.seconds()).unwrap();
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!(
            "foo  @ {:?}",
            CYCCNT::now().elapsed_since_epoch::<Microseconds<i32>>()
        )
        .unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!(
            "bar  @ {:?}",
            CYCCNT::now().elapsed_since_epoch::<Microseconds<i32>>()
        )
            .unwrap();
    }

    extern "C" {
        fn UART0();
    }
};


/// Implementation of the `Monotonic` trait based on CYCle CouNTer
pub struct CYCCNT;

impl rtic::Monotonic for CYCCNT {
    unsafe fn reset() {
        (0xE0001004 as *mut u32).write_volatile(0)
    }
}

impl time::Clock for CYCCNT {
    type Rep = i32;

    // the period of 64 MHz
    const PERIOD: time::Period = time::Period::new_raw(1, 64_000_000);

    fn now() -> Instant<Self> {
        let ticks = DWT::get_cycle_count();

        Instant::new(ticks as i32)
    }
}
