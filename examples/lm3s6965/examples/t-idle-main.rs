//! examples/t-idle-main.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::debug;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        (Shared {}, Local {})
    }

    #[idle]
    fn taskmain(_: taskmain::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        loop {
            cortex_m::asm::nop();
        }
    }
}
