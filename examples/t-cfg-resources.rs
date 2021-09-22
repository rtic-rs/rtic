//! [compile-pass] check that `#[cfg]` attributes applied on resources work
//!
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::debug;

    #[shared]
    struct Shared {
        // A conditionally compiled resource behind feature_x
        #[cfg(feature = "feature_x")]
        x: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        (
            Shared {
                #[cfg(feature = "feature_x")]
                x: 0,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
