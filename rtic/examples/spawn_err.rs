//! examples/spawn.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![deny(missing_docs)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        hprintln!("init");
        foo::spawn().unwrap();
        match foo::spawn() {
            Ok(_) => {}
            Err(()) => hprintln!("Cannot spawn a spawned (running) task!"),
        }

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_: foo::Context) {
        hprintln!("foo");

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
