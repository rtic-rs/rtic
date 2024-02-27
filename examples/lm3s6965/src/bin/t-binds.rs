//! [compile-pass] Check that `binds` works as advertised

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::debug;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        (Shared {}, Local {})
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
