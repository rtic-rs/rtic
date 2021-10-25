//! examples/lockall_soundness3.rs

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        a: u32,
        b: i64,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared { a: 1, b: 2 }, Local {}, init::Monotonics())
    }

    // when omitted priority is assumed to be `1`
    #[task(shared = [a, b])]
    fn foo(mut c: foo::Context) {
        // let s = c.shared.lock(|s| s); // lifetime error
        // hprintln!("a {}", s.a).ok();

        // let a = c.shared.lock(|foo::Shared { a, b: _ }| a); // lifetime error
        // hprintln!("a {}", a).ok();
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
