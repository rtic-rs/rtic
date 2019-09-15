//! examples/smallest.rs

#![no_main]
#![no_std]

use panic_semihosting as _; // panic handler
use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {};
