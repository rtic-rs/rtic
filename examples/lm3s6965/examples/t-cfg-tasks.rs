//! [compile-pass] check that `#[cfg]` attributes applied on tasks work

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
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

    #[cfg(feature = "feature_x")]
    #[task]
    async fn opt_sw_task(cx: opt_sw_task::Context) {}

    #[cfg(feature = "feature_x")]
    #[task(binds = UART0)]
    fn opt_hw_task(cx: opt_hw_task::Context) {}
}
