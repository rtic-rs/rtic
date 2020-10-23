//! examples/spawn.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[init]
    fn init(_c: init::Context) -> init::LateResources {
        foo::spawn(1, 2).unwrap();

        init::LateResources {}
    }

    #[task()]
    fn foo(_c: foo::Context, x: i32, y: u32) {
        hprintln!("foo {}, {}", x, y).unwrap();
        if x == 2 {
            debug::exit(debug::EXIT_SUCCESS);
        }
        foo::spawn(2, 3).unwrap();
    }
}
