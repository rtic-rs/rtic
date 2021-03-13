//! examples/minimal-early-resource.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::debug;

    #[resources]
    struct Resources {
        #[init(0)]
        resource_x: u32,
    }

    #[idle(resources = [resource_x])]
    fn idle(_: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
