//! examples/cfg.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[cfg(debug_assertions)] // <- `true` when using the `dev` profile
        #[init(0)]
        count: u32,
    }

    #[init]
    fn init(_: init::Context) {
        // ..
    }

    #[task(priority = 3, resources = [count], spawn = [log])]
    fn foo(_c: foo::Context) {
        #[cfg(debug_assertions)]
        {
            *_c.resources.count += 1;

            _c.spawn.log(*_c.resources.count).ok();
        }

        // this wouldn't compile in `release` mode
        // *resources.count += 1;

        // ..
    }

    #[cfg(debug_assertions)]
    #[task]
    fn log(_: log::Context, n: u32) {
        hprintln!(
            "foo has been called {} time{}",
            n,
            if n == 1 { "" } else { "s" }
        )
        .ok();
    }

    extern "C" {
        fn UART0();
        fn UART1();
    }
};
