//! examples/type-usage.rs

#![no_main]
#![no_std]

use panic_semihosting as _; // panic handler
use rtic::app;

#[app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    type Test = u32;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        (Shared {}, Local {}, init::Monotonics {})
    }

    #[task]
    fn t1(_: t1::Context, _val: Test) {}
}
