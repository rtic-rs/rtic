//! examples/smallest.rs

#![no_main]
#![no_std]

use panic_semihosting as _; // panic handler
use rtic::app;

#[app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    type Test = u32;

    #[task]
    fn t1(_: t1::Context, _val: Test) {}
}
