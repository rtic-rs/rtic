//! [compile-pass] Cross initialization of late resources

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(cores = 2, device = homogeneous)]
const APP: () = {
    struct Resources {
        // owned by core #1 but initialized by core #0
        x: u32,

        // owned by core #0 but initialized by core #1
        y: u32,
    }

    #[init(core = 0, late = [x])]
    fn a(_: a::Context) -> a::LateResources {
        a::LateResources { x: 0 }
    }

    #[idle(core = 0, resources = [y])]
    fn b(_: b::Context) -> ! {
        loop {}
    }

    #[init(core = 1)]
    fn c(_: c::Context) -> c::LateResources {
        c::LateResources { y: 0 }
    }

    #[idle(core = 1, resources = [x])]
    fn d(_: d::Context) -> ! {
        loop {}
    }
};
