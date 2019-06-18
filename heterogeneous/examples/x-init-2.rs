//! [compile-pass] Cross initialization of late resources

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(cores = 2, device = heterogeneous)]
const APP: () = {
    extern "C" {
        // owned by core #1 but initialized by core #0
        static mut X: u32;

        // owned by core #0 but initialized by core #1
        static mut Y: u32;
    }

    #[init(core = 0, late = [X])]
    fn a(_: a::Context) -> a::LateResources {
        a::LateResources { X: 0 }
    }

    #[idle(core = 0, resources = [Y])]
    fn b(_: b::Context) -> ! {
        loop {}
    }

    #[init(core = 1)]
    fn c(_: c::Context) -> c::LateResources {
        c::LateResources { Y: 0 }
    }

    #[idle(core = 1, resources = [X])]
    fn d(_: d::Context) -> ! {
        loop {}
    }
};
