//! examples/minmal-task-local-to-task-local.rs
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    pub struct InitType {
        x: u32,
    }

    pub struct TaskLocal {
        y: u32,
    }

    #[resources]
    struct Resources {
        #[task_local]
        #[init(InitType { x: 0 })]
        init_resource: InitType,

        #[task_local]
        local: crate::app::TaskLocal,
    }

    #[init(resources = [ init_resource ])]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        let y = cx.resources.init_resource.x;
        (
            init::LateResources {
                local: TaskLocal { y },
            },
            init::Monotonics(),
        )
    }

    // task_local is task_local
    #[idle(resources = [local])]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("IDLE:shared = {}", cx.resources.local.y).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
