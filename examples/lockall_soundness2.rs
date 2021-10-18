//! examples/lockall_soundness2.rs

// #![deny(unsafe_code)]
#![deny(warnings)]
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
        static mut X: Option<&'static mut u32> = None;
        static mut Y: u32 = 0;
        c.shared.lock(|foo::Shared { a, b }| {
            hprintln!("s.a = {}, s.b = {}", a, b).ok();
            **a += 1;

            // soundness check
            // c.shared.lock(|s| {}); // borrow error
            // c.shared.a.lock(|s| {}); // borrow error

            unsafe {
                X = Some(&mut Y);
                // X = Some(*a); // lifetime issue
                // X = Some(&mut **a); // lifetime issue
                // X = Some(&'static mut **a); // not rust
            }
            hprintln!("s.a = {}, s.b = {}", a, b).ok();
        });
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
