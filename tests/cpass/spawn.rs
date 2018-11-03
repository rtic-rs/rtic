//! Check code generation of `spawn`
#![feature(extern_crate_item_prelude)] // ???
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [foo, bar, baz])]
    fn init() {
        let _: Result<(), ()> = spawn.foo();
        let _: Result<(), u32> = spawn.bar(0);
        let _: Result<(), (u32, u32)> = spawn.baz(0, 1);
    }

    #[idle(spawn = [foo, bar, baz])]
    fn idle() -> ! {
        let _: Result<(), ()> = spawn.foo();
        let _: Result<(), u32> = spawn.bar(0);
        let _: Result<(), (u32, u32)> = spawn.baz(0, 1);

        loop {}
    }

    #[exception(spawn = [foo, bar, baz])]
    fn SVCall() {
        let _: Result<(), ()> = spawn.foo();
        let _: Result<(), u32> = spawn.bar(0);
        let _: Result<(), (u32, u32)> = spawn.baz(0, 1);
    }

    #[interrupt(spawn = [foo, bar, baz])]
    fn UART0() {
        let _: Result<(), ()> = spawn.foo();
        let _: Result<(), u32> = spawn.bar(0);
        let _: Result<(), (u32, u32)> = spawn.baz(0, 1);
    }

    #[task(spawn = [foo, bar, baz])]
    fn foo() {
        let _: Result<(), ()> = spawn.foo();
        let _: Result<(), u32> = spawn.bar(0);
        let _: Result<(), (u32, u32)> = spawn.baz(0, 1);
    }

    #[task]
    fn bar(_x: u32) {}

    #[task]
    fn baz(_x: u32, _y: u32) {}

    extern "C" {
        fn UART1();
    }
};
