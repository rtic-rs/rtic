//! examples/init.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, peripherals = true)]
mod app {
    use examples_runner::{println, exit};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init(local = [x: u32 = 0])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Cortex-M peripherals
        let _core: cortex_m::Peripherals = cx.core;

        // Device specific peripherals
        let _device: examples_runner::pac::Peripherals = cx.device;

        // Locals in `init` have 'static lifetime
        let _x: &'static mut u32 = cx.local.x;

        // Access to the critical section token,
        // to indicate that this is a critical seciton
        let _cs_token: bare_metal::CriticalSection = cx.cs;

        println!("init");

        exit();

        // (Shared {}, Local {}, init::Monotonics())
    }
}
