//! examples/baseline.rs

#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtic::time::{self, Instant};

// NOTE: does NOT properly work on QEMU
#[rtic::app(device = lm3s6965, monotonic = crate::CYCCNT, sys_timer_freq = 64_000_000)]
const APP: () = {
    #[init(spawn = [foo])]
    fn init(cx: init::Context) {
        // omitted: initialization of `CYCCNT`

        hprintln!("init(baseline = {:?})", cx.start).unwrap();

        // `foo` inherits the baseline of `init`: `Instant(0)`
        cx.spawn.foo().unwrap();
    }

    #[task(schedule = [foo])]
    fn foo(cx: foo::Context) {
        static mut ONCE: bool = true;

        hprintln!("foo(baseline = {:?})", cx.scheduled).unwrap();

        if *ONCE {
            *ONCE = false;

            rtic::pend(Interrupt::UART0);
        } else {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[task(binds = UART0, spawn = [foo])]
    fn uart0(cx: uart0::Context) {
        hprintln!("UART0(baseline = {:?})", cx.start).unwrap();

        // `foo` inherits the baseline of `UART0`: its `start` time
        cx.spawn.foo().unwrap();
    }

    extern "C" {
        fn UART1();
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
