//! examples/multilock_vs_lockall.rs

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
    fn locks(mut c: locks::Context) {
        // lock-all structure
        c.shared.lock(|s| {
            *s.shared1 += 1;
            *s.shared2 += 2;
            *s.shared3 += 3;
            hprintln!(
                "Multiple locks, s1: {}, s2: {}, s3: {}",
                s.shared1,
                s.shared2,
                s.shared3
            )
            .ok();
        });

        // lock-all destructure
        c.shared.lock(
            |locks::Shared {
                 shared1,
                 shared2,
                 shared3,
             }| {
                **shared1 += 1;
                **shared2 += 2;
                **shared3 += 3;
                hprintln!(
                    "Multiple locks, s1: {}, s2: {}, s3: {}",
                    shared1,
                    shared2,
                    shared3
                )
                .ok();
            },
        );

        // nested multi-lock
        let s1 = c.shared.shared1;
        let s2 = c.shared.shared2;
        let s3 = c.shared.shared3;

        (s1, s2, s3).lock(|s1, s2, s3| {
            *s1 += 1;
            *s2 += 2;
            *s3 += 3;

            hprintln!("Multiple locks, s1: {}, s2: {}, s3: {}", s1, s2, s3).ok();
        });

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
