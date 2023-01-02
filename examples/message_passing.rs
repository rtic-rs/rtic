//! examples/message_passing.rs

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
        foo::spawn(1, 1).unwrap();
        foo::spawn(1, 2).unwrap();
        foo::spawn(2, 3).unwrap();
        assert!(foo::spawn(1, 4).is_err()); // The capacity of `foo` is reached

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(capacity = 3)]
    fn foo(_c: foo::Context, x: i32, y: u32) {
        hprintln!("foo {}, {}", x, y).unwrap();
        if x == 2 {
            debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        }
    }
}
