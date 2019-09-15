//! [compile-pass] Check `schedule` code generation

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;
use rtfm::cyccnt::{Instant, U32Ext as _};

#[rtfm::app(device = lm3s6965, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    #[init(schedule = [foo, bar, baz])]
    fn init(c: init::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.start + 10.cycles());
        let _: Result<(), u32> = c.schedule.bar(c.start + 20.cycles(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.start + 30.cycles(), 0, 1);
    }

    #[idle(schedule = [foo, bar, baz])]
    fn idle(c: idle::Context) -> ! {
        let _: Result<(), ()> = c.schedule.foo(Instant::now() + 40.cycles());
        let _: Result<(), u32> = c.schedule.bar(Instant::now() + 50.cycles(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(Instant::now() + 60.cycles(), 0, 1);

        loop {}
    }

    #[task(binds = SVCall, schedule = [foo, bar, baz])]
    fn svcall(c: svcall::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.start + 70.cycles());
        let _: Result<(), u32> = c.schedule.bar(c.start + 80.cycles(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.start + 90.cycles(), 0, 1);
    }

    #[task(binds = UART0, schedule = [foo, bar, baz])]
    fn uart0(c: uart0::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.start + 100.cycles());
        let _: Result<(), u32> = c.schedule.bar(c.start + 110.cycles(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.start + 120.cycles(), 0, 1);
    }

    #[task(schedule = [foo, bar, baz])]
    fn foo(c: foo::Context) {
        let _: Result<(), ()> = c.schedule.foo(c.scheduled + 130.cycles());
        let _: Result<(), u32> = c.schedule.bar(c.scheduled + 140.cycles(), 0);
        let _: Result<(), (u32, u32)> = c.schedule.baz(c.scheduled + 150.cycles(), 0, 1);
    }

    #[task]
    fn bar(_: bar::Context, _x: u32) {}

    #[task]
    fn baz(_: baz::Context, _x: u32, _y: u32) {}

    extern "C" {
        fn UART1();
    }
};
