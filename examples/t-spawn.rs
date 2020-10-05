//! [compile-pass] Check code generation of `spawn`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[init(spawn = [foo, bar, baz])]
    fn init(c: init::Context) {
        let _: Result<(), ()> = c.spawn.foo();
        let _: Result<(), u32> = c.spawn.bar(0);
        let _: Result<(), (u32, u32)> = c.spawn.baz(0, 1);
    }

    #[idle(spawn = [foo, bar, baz])]
    fn idle(c: idle::Context) -> ! {
        let _: Result<(), ()> = c.spawn.foo();
        let _: Result<(), u32> = c.spawn.bar(0);
        let _: Result<(), (u32, u32)> = c.spawn.baz(0, 1);

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = SVCall, spawn = [foo, bar, baz])]
    fn svcall(c: svcall::Context) {
        let _: Result<(), ()> = c.spawn.foo();
        let _: Result<(), u32> = c.spawn.bar(0);
        let _: Result<(), (u32, u32)> = c.spawn.baz(0, 1);
    }

    #[task(binds = UART0, spawn = [foo, bar, baz])]
    fn uart0(c: uart0::Context) {
        let _: Result<(), ()> = c.spawn.foo();
        let _: Result<(), u32> = c.spawn.bar(0);
        let _: Result<(), (u32, u32)> = c.spawn.baz(0, 1);
    }

    #[task(spawn = [foo, bar, baz])]
    fn foo(c: foo::Context) {
        let _: Result<(), ()> = c.spawn.foo();
        let _: Result<(), u32> = c.spawn.bar(0);
        let _: Result<(), (u32, u32)> = c.spawn.baz(0, 1);
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
