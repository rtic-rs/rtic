//! [compile-pass] Check `schedule` code generation

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT)]
mod app {
    use rtic::cyccnt::{Instant, U32Ext as _};

    #[init]
    fn init(c: init::Context) -> init::LateResources {
        let _: Result<(), ()> = foo::schedule(c.start + 10.cycles());
        let _: Result<(), u32> = bar::schedule(c.start + 20.cycles(), 0);
        let _: Result<(), (u32, u32)> = baz::schedule(c.start + 30.cycles(), 0, 1);

        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let _: Result<(), ()> = foo::schedule(Instant::now() + 40.cycles());
        let _: Result<(), u32> = bar::schedule(Instant::now() + 50.cycles(), 0);
        let _: Result<(), (u32, u32)> = baz::schedule(Instant::now() + 60.cycles(), 0, 1);

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = SVCall)]
    fn svcall(c: svcall::Context) {
        let _: Result<(), ()> = foo::schedule(c.start + 70.cycles());
        let _: Result<(), u32> = bar::schedule(c.start + 80.cycles(), 0);
        let _: Result<(), (u32, u32)> = baz::schedule(c.start + 90.cycles(), 0, 1);
    }

    #[task(binds = UART0)]
    fn uart0(c: uart0::Context) {
        let _: Result<(), ()> = foo::schedule(c.start + 100.cycles());
        let _: Result<(), u32> = bar::schedule(c.start + 110.cycles(), 0);
        let _: Result<(), (u32, u32)> = baz::schedule(c.start + 120.cycles(), 0, 1);
    }

    #[task]
    fn foo(c: foo::Context) {
        let _: Result<(), ()> = foo::schedule(c.scheduled + 130.cycles());
        let _: Result<(), u32> = bar::schedule(c.scheduled + 140.cycles(), 0);
        let _: Result<(), (u32, u32)> = baz::schedule(c.scheduled + 150.cycles(), 0, 1);
    }

    #[task]
    fn bar(_: bar::Context, _x: u32) {}

    #[task]
    fn baz(_: baz::Context, _x: u32, _y: u32) {}

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}
