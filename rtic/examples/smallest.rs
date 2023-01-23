//! examples/smallest.rs

#![no_main]
#![no_std]
#![deny(missing_docs)]

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
    fn init(_: init::Context) -> (Shared, Local) {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        (Shared {}, Local {})
    }
}
