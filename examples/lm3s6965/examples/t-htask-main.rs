//! examples/t-task-main.rs

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
        rtic::pend(lm3s6965::Interrupt::UART0);

        (Shared {}, Local {})
    }

    #[task(binds = UART0)]
    fn taskmain(_: taskmain::Context) {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
