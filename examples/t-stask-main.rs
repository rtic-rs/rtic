#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::debug;

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        taskmain::spawn().ok();

        (init::LateResources {}, init::Monotonics())
    }

    #[task]
    fn taskmain(_: taskmain::Context) {
        debug::exit(debug::EXIT_SUCCESS);
    }
}
