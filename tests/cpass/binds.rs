//! Check that `binds` works as advertised
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init() {}

    #[exception(binds = SVCall)]
    fn foo() {}

    #[interrupt(binds = UART0)]
    fn bar() {}
};

fn foo_trampoline(_: foo::Context) {}

fn bar_trampoline(_: bar::Context) {}
