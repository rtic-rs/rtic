#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::{app, Instant};

#[app(device = lm3s6965)]
const APP: () = {
    #[init(schedule = [foo, bar, baz])]
    fn init() {
        let _: Result<(), ()> = schedule.foo(start + 10.cycles());
        let _: Result<(), u32> = schedule.bar(start + 20.cycles(), 0);
        let _: Result<(), (u32, u32)> = schedule.baz(start + 30.cycles(), 0, 1);
    }

    #[idle(schedule = [foo, bar, baz])]
    fn idle() -> ! {
        let _: Result<(), ()> = schedule.foo(Instant::now() + 40.cycles());
        let _: Result<(), u32> = schedule.bar(Instant::now() + 50.cycles(), 0);
        let _: Result<(), (u32, u32)> = schedule.baz(Instant::now() + 60.cycles(), 0, 1);

        loop {}
    }

    #[exception(schedule = [foo, bar, baz])]
    fn SVCall() {
        let _: Result<(), ()> = schedule.foo(start + 70.cycles());
        let _: Result<(), u32> = schedule.bar(start + 80.cycles(), 0);
        let _: Result<(), (u32, u32)> = schedule.baz(start + 90.cycles(), 0, 1);
    }

    #[interrupt(schedule = [foo, bar, baz])]
    fn UART0() {
        let _: Result<(), ()> = schedule.foo(start + 100.cycles());
        let _: Result<(), u32> = schedule.bar(start + 110.cycles(), 0);
        let _: Result<(), (u32, u32)> = schedule.baz(start + 120.cycles(), 0, 1);
    }

    #[task(schedule = [foo, bar, baz])]
    fn foo() {
        let _: Result<(), ()> = schedule.foo(scheduled + 130.cycles());
        let _: Result<(), u32> = schedule.bar(scheduled + 140.cycles(), 0);
        let _: Result<(), (u32, u32)> = schedule.baz(scheduled + 150.cycles(), 0, 1);
    }

    #[task]
    fn bar(_x: u32) {}

    #[task]
    fn baz(_x: u32, _y: u32) {}

    extern "C" {
        fn UART1();
    }
};
