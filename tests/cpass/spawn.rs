//! Check code generation of `spawn`
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
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

        loop {}
    }

    #[exception(spawn = [foo, bar, baz])]
    fn SVCall(c: SVCall::Context) {
        let _: Result<(), ()> = c.spawn.foo();
        let _: Result<(), u32> = c.spawn.bar(0);
        let _: Result<(), (u32, u32)> = c.spawn.baz(0, 1);
    }

    #[interrupt(spawn = [foo, bar, baz])]
    fn UART0(c: UART0::Context) {
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

    extern "C" {
        fn UART1();
    }
};
