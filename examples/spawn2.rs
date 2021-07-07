//! examples/spawn2.rs

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
        foo::spawn(1, 2).unwrap();

        (Shared {}, Local {}, init::Monotonics {})
    }

    #[task]
    fn foo(_c: foo::Context, x: i32, y: u32) {
        hprintln!("foo {}, {}", x, y).unwrap();
        if x == 2 {
            debug::exit(debug::EXIT_SUCCESS);
        }
        foo2::spawn(2).unwrap();
    }

    #[task]
    fn foo2(_c: foo2::Context, x: i32) {
        hprintln!("foo2 {}", x).unwrap();
        foo::spawn(x, 0).unwrap();
    }
}
