#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::exit;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        assert!(cortex_m::Peripherals::take().is_none());
        exit();

        // (Shared {}, Local {}, init::Monotonics())
    }
}
