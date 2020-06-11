#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::debug;
use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [main])]
    fn init(cx: init::Context) {
        cx.spawn.main().ok();
    }

    #[task]
    fn main(_: main::Context) {
        debug::exit(debug::EXIT_SUCCESS);
    }

    extern "C" {
        fn UART0();
    }
};
