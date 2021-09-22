//! examples/capacity.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(binds = UART0)]
    fn uart0(_: uart0::Context) {
        foo::spawn(0).unwrap();
        foo::spawn(1).unwrap();
        foo::spawn(2).unwrap();
        foo::spawn(3).unwrap();

        bar::spawn().unwrap();
    }

    #[task(capacity = 4)]
    fn foo(_: foo::Context, x: u32) {
        hprintln!("foo({})", x).unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!("bar").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
