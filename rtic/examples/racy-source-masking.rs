//! Bug in sourcemasking test.

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        shared1: u32, //shared between foo and bar, masks GPIOA and GPIOB
        shared2: u32, //same
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();

        (
            Shared {
                shared1: 0,
                shared2: 0,
            },
            Local {},
        )
    }

    #[task(shared = [shared1, shared2])]
    async fn foo(c: foo::Context) {
        let mut shared1 = c.shared.shared1;
        let mut shared2 = c.shared.shared2;

        shared2.lock(|shared2| {
            hprintln!("foo accessing shared2 resource start");
            //GPIOA and GPIOB masked
            *shared2 += 1; //OK access to shared 1
            shared1.lock(|shared1| {
                //GPIOA and GPIOB masked again
                *shared1 += 1; //so far so ggood
            });

            hprintln!("pending bar");
            rtic::pend(lm3s6965::Interrupt::GPIOB);
            hprintln!("bar pended");

            //GPIOA and GPIOB unmasked
            //racy access to shared2!
            *shared2 += 1;
            hprintln!("foo accessing shared2 resource end");
        });

        //GPIOA and GPIOB unmasked again

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(binds = GPIOB, priority = 2, shared = [shared1, shared2])]
    fn bar(mut c: bar::Context) {
        hprintln!("bar running");
        c.shared.shared2.lock(|shared2| {
            hprintln!("bar accesing shared2 resource");
            *shared2 += 1; // this can race with access in foo!
        });
    }
}
