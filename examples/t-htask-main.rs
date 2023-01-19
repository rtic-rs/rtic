//! examples/h-task-main.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![deny(missing_docs)]
#![no_main]
#![no_std]

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
