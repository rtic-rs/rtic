//! [compile-pass] Split initialization of late resources

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(cores = 2, device = heterogeneous)]
const APP: () = {
    struct Resources {
        x: u32,
        y: u32,
    }

    #[init(core = 0, late = [x])]
    fn a(_: a::Context) -> a::LateResources {
        a::LateResources { x: 0 }
    }

    #[init(core = 1)]
    fn b(_: b::Context) -> b::LateResources {
        b::LateResources { y: 0 }
    }
};
