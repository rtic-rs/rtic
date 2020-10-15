//! [compile-pass] Check code generation of `spawn`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[init]
    fn init(_: init::Context) -> init::LateResources {
        let _: Result<(), ()> = foo::spawn();
        let _: Result<(), u32> = bar::spawn(0);
        let _: Result<(), (u32, u32)> = baz::spawn(0, 1);

        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let _: Result<(), ()> = foo::spawn();
        let _: Result<(), u32> = bar::spawn(0);
        let _: Result<(), (u32, u32)> = baz::spawn(0, 1);

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = SVCall)]
    fn svcall(_: svcall::Context) {
        let _: Result<(), ()> = foo::spawn();
        let _: Result<(), u32> = bar::spawn(0);
        let _: Result<(), (u32, u32)> = baz::spawn(0, 1);
    }

    #[task(binds = UART0)]
    fn uart0(_: uart0::Context) {
        let _: Result<(), ()> = foo::spawn();
        let _: Result<(), u32> = bar::spawn(0);
        let _: Result<(), (u32, u32)> = baz::spawn(0, 1);
    }

    #[task]
    fn foo(_: foo::Context) {
        let _: Result<(), ()> = foo::spawn();
        let _: Result<(), u32> = bar::spawn(0);
        let _: Result<(), (u32, u32)> = baz::spawn(0, 1);
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
