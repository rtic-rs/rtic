//! examples/idle.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {

    #[resources]
    struct Resources {
        #[init(0)]
        resource_x: u32,
    }

    #[idle(resources = [resource_x])]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
