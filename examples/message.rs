//! examples/message.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        foo::spawn(/* no message */).unwrap();

        init::LateResources {}
    }

    #[task]
    fn foo(_: foo::Context) {
        static mut COUNT: u32 = 0;

        hprintln!("foo").unwrap();

        bar::spawn(*COUNT).unwrap();
        *COUNT += 1;
    }

    #[task]
    fn bar(_: bar::Context, x: u32) {
        hprintln!("bar({})", x).unwrap();

        baz::spawn(x + 1, x + 2).unwrap();
    }

    #[task]
    fn baz(_: baz::Context, x: u32, y: u32) {
        hprintln!("baz({}, {})", x, y).unwrap();

        if x + y > 4 {
            debug::exit(debug::EXIT_SUCCESS);
        }

        foo::spawn().unwrap();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}
