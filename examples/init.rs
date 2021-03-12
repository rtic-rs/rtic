//! examples/init.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        // Cortex-M peripherals
        let _core: cortex_m::Peripherals = cx.core;

        // Device specific peripherals
        let _device: lm3s6965::Peripherals = cx.device;

        // Access to the critical section token,
        // to indicate that this is a critical seciton
        let _cs_token: bare_metal::CriticalSection = cx.cs;

        hprintln!("init").unwrap();

        debug::exit(debug::EXIT_SUCCESS);

        (init::LateResources {}, init::Monotonics())
    }
}
