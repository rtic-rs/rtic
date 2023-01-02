//! examples/spawn.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("init").unwrap();
        foo::spawn().unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
