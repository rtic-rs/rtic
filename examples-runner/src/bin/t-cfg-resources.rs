//! [compile-pass] check that `#[cfg]` attributes applied on resources work
//!
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::exit;

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
        exit();
        // loop {
        //     cortex_m::asm::nop();
        // }
    }
}
