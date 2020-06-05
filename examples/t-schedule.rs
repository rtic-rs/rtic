//! [compile-pass] Check `schedule` code generation

#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;
use cortex_m::peripheral::DWT;
use rtic::time::{self, instant::Instant, prelude::*};

#[rtic::app(device = lm3s6965, monotonic = crate::CYCCNT, sys_timer_freq = 64_000_000)]
const APP: () = {
    #[init(schedule = [foo, bar, baz])]
    fn init(c: init::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.start + 10.milliseconds());
        let _: Result<(), u32> = c.schedule.bar(c.start + 20.milliseconds(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.start + 30.milliseconds(), 0, 1);
    }

    #[idle(schedule = [foo, bar, baz])]
    fn idle(c: idle::Context) -> ! {
        let _: Result<(), ()> = c.schedule.foo(CYCCNT::now() + 40.milliseconds());
        let _: Result<(), u32> = c.schedule.bar(CYCCNT::now() + 50.milliseconds(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(CYCCNT::now() + 60.milliseconds(), 0, 1);

        loop {}
    }

    #[task(binds = SVCall, schedule = [foo, bar, baz])]
    fn svcall(c: svcall::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.start + 70.milliseconds());
        let _: Result<(), u32> = c.schedule.bar(c.start + 80.milliseconds(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.start + 90.milliseconds(), 0, 1);
    }

    #[task(binds = UART0, schedule = [foo, bar, baz])]
    fn uart0(c: uart0::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.start + 100.milliseconds());
        let _: Result<(), u32> = c.schedule.bar(c.start + 110.milliseconds(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.start + 120.milliseconds(), 0, 1);
    }

    #[task(schedule = [foo, bar, baz])]
    fn foo(c: foo::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.scheduled + 130.milliseconds());
        let _: Result<(), u32> = c.schedule.bar(c.scheduled + 140.milliseconds(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.scheduled + 150.milliseconds(), 0, 1);
    }

    #[task]
    fn bar(_: bar::Context, _x: u32) {}

    #[task]
    fn baz(_: baz::Context, _x: u32, _y: u32) {}

    extern "C" {
        fn UART1();
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