//! examples/task-local_minimal.rs
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[resources]
    struct Resources {
        // A local (move), late resource
        #[task_local]
        l: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        init::LateResources { l: 42 }
    }

    // l is task_local
    #[idle(resources =[l])]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("IDLE:l = {}", cx.resources.l).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
