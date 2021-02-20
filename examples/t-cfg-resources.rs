//! [compile-pass] check that `#[cfg]` attributes applied on resources work
//!
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[resources]
    struct Resources {
        // A resource
        #[init(0)]
        shared: u32,
        // A conditionally compiled resource behind feature_x
        #[cfg(feature = "feature_x")]
        x: u32,
        dummy: (), // dummy such that we have at least one late resource
    }
    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (
            init::LateResources {
                // The feature needs to be applied everywhere x is defined or used
                #[cfg(feature = "feature_x")]
                x: 0,
                dummy: (), // dummy such that we have at least one late resource
            },
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
