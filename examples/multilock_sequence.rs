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
    fn locks(c: locks::Context) {
        // nested multi-lock
        let mut s1 = c.shared.shared1;
        let mut s2 = c.shared.shared2;
        let mut s3 = c.shared.shared3;

        (&mut s1, &mut s2, &mut s3).lock(|s1, s2, s3| {
            *s1 += 1;
            *s2 += 2;
            *s3 += 3;

            hprintln!("Multiple locks, s1: {}, s2: {}, s3: {}", s1, s2, s3).ok();
        });

        // re-construct
        let s = locks::SharedResources {
            shared1: s1,
            shared2: s2,
            shared3: s3,
        };

        // second nested multi-lock destruct
        let locks::SharedResources {
            shared1,
            shared2,
            shared3,
        } = s;

        (shared1, shared2, shared3).lock(|s1, s2, s3| {
            *s1 += 1;
            *s2 += 2;
            *s3 += 3;

            hprintln!("Multiple locks, s1: {}, s2: {}, s3: {}", s1, s2, s3).ok();
        });

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
