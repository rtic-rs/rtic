//! examples/spawn_loop.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
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

        (Shared {}, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        for _ in 0..3 {
            foo::spawn().unwrap();
            hprintln!("idle");
        }
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        loop {}
    }

    #[task(priority = 1)]
    async fn foo(_: foo::Context) {
        hprintln!("foo");
    }
}
