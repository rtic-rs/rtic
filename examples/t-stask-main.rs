#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::debug;
use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[init(spawn = [main])]
    fn init(cx: init::Context) {
        cx.spawn.main().ok();
    }

    #[task]
    fn main(_: main::Context) {
        debug::exit(debug::EXIT_SUCCESS);
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}
