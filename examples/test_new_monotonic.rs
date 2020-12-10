//! examples/test_new_monotonic.rs

#![no_main]
#![no_std]

use panic_semihosting as _; // panic handler
use rtic::app;

#[app(device = lm3s6965, dispatchers = [UART])]
mod app {
    #[monotonic(binds = SomeISR1)]
    type MyMono1 = hal::Mono1;

    #[monotonic(binds = SomeISR2, default = true)]
    type MyMono2 = hal::Mono2;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
    }

    #[task]
    fn task1(_: task1::Context) {}

    #[task]
    fn task2(_: task2::Context) {}
}

