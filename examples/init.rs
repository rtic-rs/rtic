//! examples/init.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init(local = [x: u32 = 0])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Cortex-M peripherals
        let _core: cortex_m::Peripherals = cx.core;

        // Device specific peripherals
        let _device: lm3s6965::Peripherals = cx.device;

        // Locals in `init` have 'static lifetime
        let _x: &'static mut u32 = cx.local.x;

        // Access to the critical section token,
        // to indicate that this is a critical seciton
        let _cs_token: bare_metal::CriticalSection = cx.cs;

        hprintln!("init").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        (Shared {}, Local {}, init::Monotonics())
    }
}
