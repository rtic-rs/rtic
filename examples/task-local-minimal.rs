//! examples/task-local_minimal.rs
#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[resources]
    struct Resources {
        // A local (move), late resource
        #[task_local]
        task_local: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (init::LateResources { task_local: 42 }, init::Monotonics())
    }

    // task_local is task_local
    #[idle(resources =[task_local])]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("IDLE:l = {}", cx.resources.task_local).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
