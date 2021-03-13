//! examples/minimal-task-local-type-late.rs
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
        late: crate::app::TaskLocal,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (
            init::LateResources {
                late: TaskLocal { x: 42 },
            },
            init::Monotonics(),
        )
    }

    // task_local is task_local
    #[idle(resources = [late])]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("IDLE:late = {}", cx.resources.late.x).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
