//! examples/smallest.rs

#![no_main]
#![no_std]

use examples_runner as _; // panic handler
use rtic::app;

#[app(device = examples_runner::pac)]
mod app {
    use examples_runner::exit;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        exit();
        // (Shared {}, Local {}, init::Monotonics())
    }
}
