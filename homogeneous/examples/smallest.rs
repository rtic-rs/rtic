#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(cores = 2, device = homogeneous)]
const APP: () = {};
