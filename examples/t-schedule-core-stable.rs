//! [compile-pass] Check `schedule` code generation

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    #[init]
    fn init(c: init::Context) -> (init::LateResources, init::Monotonics) {
        let _c: cortex_m::Peripherals = c.core;

        (init::LateResources {}, init::Monotonics())
    }

    #[task]
    fn some_task(_: some_task::Context) {}
}
