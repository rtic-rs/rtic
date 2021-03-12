//! examples/minimal-task-local-late.rs
#![deny(unsafe_code)]
#![deny(warnings)]
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
        late: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (init::LateResources { late: 42 }, init::Monotonics())
    }

    // task_local is task_local
    #[idle(resources = [late])]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("IDLE:late = {}", cx.resources.late).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
