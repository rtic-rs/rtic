//! examples/smallest.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use core::panic::PanicInfo;
use cortex_m_semihosting::debug;
use rtic::app;

#[app(device = lm3s6965)]
mod app {
    use super::*;

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

#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    debug::exit(debug::EXIT_FAILURE);
    loop {}
}
