//! examples/lock-free.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        #[lock_free] // <- lock-free shared resource
        counter: u64,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        rtic::pend(Interrupt::UART0);

        (Shared { counter: 0 }, Local {})
    }

    #[task(binds = UART0, shared = [counter])] // <- same priority
    fn foo(c: foo::Context) {
        rtic::pend(Interrupt::UART1);

        *c.shared.counter += 1; // <- no lock API required
        let counter = *c.shared.counter;
        hprintln!("  foo = {}", counter);
    }

    #[task(binds = UART1, shared = [counter])] // <- same priority
    fn bar(c: bar::Context) {
        rtic::pend(Interrupt::UART0);
        *c.shared.counter += 1; // <- no lock API required
        let counter = *c.shared.counter;
        hprintln!("  bar = {}", counter);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
