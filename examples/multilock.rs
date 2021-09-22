//! examples/mutlilock.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        shared1: u32,
        shared2: u32,
        shared3: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        locks::spawn().unwrap();

        (
            Shared {
                shared1: 0,
                shared2: 0,
                shared3: 0,
            },
            Local {},
            init::Monotonics(),
        )
    }

    // when omitted priority is assumed to be `1`
    #[task(shared = [shared1, shared2, shared3])]
    fn locks(c: locks::Context) {
        let s1 = c.shared.shared1;
        let s2 = c.shared.shared2;
        let s3 = c.shared.shared3;

        (s1, s2, s3).lock(|s1, s2, s3| {
            *s1 += 1;
            *s2 += 1;
            *s3 += 1;

            hprintln!("Multiple locks, s1: {}, s2: {}, s3: {}", *s1, *s2, *s3).unwrap();
        });

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
