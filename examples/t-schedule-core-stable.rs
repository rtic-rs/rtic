//! [compile-pass] Check `schedule` code generation

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT)]
mod app {
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        let _c: rtic::Peripherals = c.core;

        init::LateResources {}
    }

    #[task]
    fn some_task(_: some_task::Context) {}

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}
