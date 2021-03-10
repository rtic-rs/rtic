//! examples/minimal_late_resource.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {

    #[resources]
    struct Resources {
        resource_x: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (init::LateResources { resource_x: 0 }, init::Monotonics {})
    }

    #[idle(resources = [resource_x])]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
