//! examples/minimal-task-local-early.rs

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
        shared: u32,

        #[task_local]
        #[init(42)]
        init_resource: u32,
    }

    #[init(resources = [ init_resource ])]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        let ir = *cx.resources.init_resource;
        (init::LateResources { shared: ir }, init::Monotonics())
    }

    // task_local is task_local
    #[idle(resources = [shared])]
    fn idle(mut cx: idle::Context) -> ! {
        hprintln!("IDLE:shared = {}", cx.resources.shared.lock(|s| *s)).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
