//! Check code generation of `unsafe` `init` / `idle` / `exception` / `interrupt` / `task`
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

unsafe fn foo() {}

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    unsafe fn init() {
        foo();
    }

    #[idle]
    unsafe fn idle() -> ! {
        foo();

        loop {}
    }

    #[exception]
    unsafe fn SVCall() {
        foo();
    }

    #[interrupt]
    unsafe fn UART0() {
        foo();
    }

    #[task]
    unsafe fn bar() {
        foo();
    }

    extern "C" {
        fn UART1();
    }
};
