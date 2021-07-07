//! examples/mutlilock.rs
//!
//! The multi-lock feature example.

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

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
        rtic::pend(Interrupt::GPIOA);

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
    #[task(binds = GPIOA, shared = [shared1, shared2, shared3])]
    fn locks(c: locks::Context) {
        let mut s1 = c.shared.shared1;
        let mut s2 = c.shared.shared2;
        let mut s3 = c.shared.shared3;

        hprintln!("Multiple single locks").unwrap();
        s1.lock(|s1| {
            s2.lock(|s2| {
                s3.lock(|s3| {
                    *s1 += 1;
                    *s2 += 1;
                    *s3 += 1;

                    hprintln!(
                        "Multiple single locks, s1: {}, s2: {}, s3: {}",
                        *s1,
                        *s2,
                        *s3
                    )
                    .unwrap();
                })
            })
        });

        hprintln!("Multilock!").unwrap();

        (s1, s2, s3).lock(|s1, s2, s3| {
            *s1 += 1;
            *s2 += 1;
            *s3 += 1;

            hprintln!(
                "Multiple single locks, s1: {}, s2: {}, s3: {}",
                *s1,
                *s2,
                *s3
            )
            .unwrap();
        });

        debug::exit(debug::EXIT_SUCCESS);
    }
}
