//! [compile-pass] Check that `binds` works as advertised

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {}

    // Cortex-M exception
    #[task(binds = SVCall)]
    fn foo(c: foo::Context) {
        foo_trampoline(c)
    }

    // LM3S6965 interrupt
    #[task(binds = UART0)]
    fn bar(c: bar::Context) {
        bar_trampoline(c)
    }
};

#[allow(dead_code)]
fn foo_trampoline(_: foo::Context) {}

#[allow(dead_code)]
fn bar_trampoline(_: bar::Context) {}
