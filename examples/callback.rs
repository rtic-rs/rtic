//! examples/callback.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use super::*;
    #[init()]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        hprintln!("init").unwrap();
        driver(&bar::spawn);
        foo::spawn(123).unwrap();
        (init::LateResources {}, init::Monotonics())
    }

    #[task()]
    fn foo(_: foo::Context, data: u32) {
        hprintln!("foo {}", data).unwrap();
        bar::spawn().unwrap();
    }

    #[task()]
    fn bar(_: bar::Context) {
        hprintln!("bar").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }
}

// some external code (e.g. driver)
fn driver<E>(_callback: &dyn Fn() -> Result<(), E>) {
    hprintln!("driver").unwrap();
}
