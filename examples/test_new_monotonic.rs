//! examples/test_new_monotonic.rs

#![no_main]
#![no_std]

use panic_semihosting as _; // panic handler
use rtic::app;

#[app(device = lm3s6965)]
mod app {
    #[monotonic(binds = SomeISR1)]
    type Mono1 = hal::Mono1;

    #[monotonic(binds = SomeISR2)]
    type Mono2 = hal::Mono2;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
    }
}

