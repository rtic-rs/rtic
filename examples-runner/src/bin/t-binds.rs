//! [compile-pass] Check that `binds` works as advertised

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::exit;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        exit();

        // (Shared {}, Local {}, init::Monotonics())
    }

    // Cortex-M exception
    #[task(binds = SVCall)]
    fn foo(c: foo::Context) {
        crate::foo_trampoline(c)
    }

    // LM3S6965 interrupt
    #[task(binds = UART0)]
    fn bar(c: bar::Context) {
        crate::bar_trampoline(c)
    }
}

#[allow(dead_code)]
fn foo_trampoline(_: app::foo::Context) {}

#[allow(dead_code)]
fn bar_trampoline(_: app::bar::Context) {}
