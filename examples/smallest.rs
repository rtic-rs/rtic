//! examples/smallest.rs

#![no_main]
#![no_std]

use panic_semihosting as _; // panic handler
use rtic::app;

#[app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::debug;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        (Shared {}, Local {}, init::Monotonics())
    }
}
