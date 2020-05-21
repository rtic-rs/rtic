//! examples/schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use cortex_m_semihosting::hprintln;
use panic_halt as _;
use rtic::time::{prelude::*, time_units::*, Ratio};

use rtic::cyccnt::CYCCNT as SysClock;

// NOTE: does NOT work on QEMU!
#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT, sys_timer_freq = 64_000_000)]
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
            SysClock::now().elapsed_since_epoch::<Microseconds<i32>>()
        )
        .unwrap();

        // Schedule `foo` to run 8e6 cycles (clock cycles) in the future
        cx.schedule.foo(now + 1.seconds()).unwrap();

        // Schedule `bar` to run 4e6 cycles in the future
        cx.schedule.bar(now + 2.seconds()).unwrap();
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!(
            "foo  @ {:?}",
            SysClock::now().elapsed_since_epoch::<Microseconds<i32>>()
        )
        .unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!(
            "bar  @ {:?}",
            SysClock::now().elapsed_since_epoch::<Microseconds<i32>>()
        )
        .unwrap();
    }

    extern "C" {
        fn UART0();
    }
};
