//! examples/extern_spawn.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

// Free function implementing the spawnable task `foo`.
fn foo(_c: app::foo::Context, x: i32, y: u32) {
    hprintln!("foo {}, {}", x, y).unwrap();
    if x == 2 {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
    app::foo::spawn(2, 3).unwrap();
}

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use crate::foo;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn(1, 2).unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    extern "Rust" {
        #[task()]
        fn foo(_c: foo::Context, _x: i32, _y: u32);
    }
}
