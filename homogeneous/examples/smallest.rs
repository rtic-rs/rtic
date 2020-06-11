#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(cores = 2, device = homogeneous)]
const APP: () = {};
