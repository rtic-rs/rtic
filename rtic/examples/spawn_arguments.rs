//! examples/spawn_arguments.rs

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
        foo::spawn(1, 1).unwrap();
        assert!(foo::spawn(1, 4).is_err()); // The capacity of `foo` is reached

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_c: foo::Context, x: i32, y: u32) {
        hprintln!("foo {}, {}", x, y);
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
