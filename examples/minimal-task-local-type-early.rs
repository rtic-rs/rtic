//! examples/minimal-task-local-type-early.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    pub struct TaskLocal {
        x: u32,
    }
    #[resources]
    struct Resources {
        // A local (move), late resource
        #[task_local]
        #[init(TaskLocal {x : 42 })]
        early: TaskLocal,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (init::LateResources {}, init::Monotonics())
    }

    // task_local is task_local
    #[idle(resources = [early])]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("IDLE:l = {}", cx.resources.early.x).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
