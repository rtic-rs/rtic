//! examples/task_named_main.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        main::spawn().unwrap();

        init::LateResources {}
    }

    #[task]
    fn main(_: main::Context) {
        hprintln!("This task is named main, useful for rust-analyzer").unwrap();
        debug::exit(debug::EXIT_SUCCESS);
    }
}
