//! examples/async-timeout.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;
use rtic_monotonics::systick::prelude::*;
systick_monotonic!(Mono, 100);

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use super::*;
    use futures::{future::FutureExt, select_biased};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    // ANCHOR: init
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init");

        Mono::start(cx.core.SYST, 12_000_000);
        // ANCHOR_END: init

        foo::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        // ANCHOR: select_biased
        // Call hal with short relative timeout using `select_biased`
        select_biased! {
            v = hal_get(1).fuse() => hprintln!("hal returned {}", v),
            _ = Mono::delay(200.millis()).fuse() =>  hprintln!("timeout", ), // this will finish first
        }

        // Call hal with long relative timeout using `select_biased`
        select_biased! {
            v = hal_get(1).fuse() => hprintln!("hal returned {}", v), // hal finish first
            _ = Mono::delay(1000.millis()).fuse() =>  hprintln!("timeout", ),
        }
        // ANCHOR_END: select_biased

        // ANCHOR: timeout_after_basic
        // Call hal with long relative timeout using monotonic `timeout_after`
        match Mono::timeout_after(1000.millis(), hal_get(1)).await {
            Ok(v) => hprintln!("hal returned {}", v),
            _ => hprintln!("timeout"),
        }
        // ANCHOR_END: timeout_after_basic

        // ANCHOR: timeout_at_basic
        // get the current time instance
        let mut instant = Mono::now();

        // do this 3 times
        for n in 0..3 {
            // absolute point in time without drift
            instant += 1000.millis();
            Mono::delay_until(instant).await;

            // absolute point in time for timeout
            let timeout = instant + 500.millis();
            hprintln!("now is {:?}, timeout at {:?}", Mono::now(), timeout);

            match Mono::timeout_at(timeout, hal_get(n)).await {
                Ok(v) => hprintln!("hal returned {} at time {:?}", v, Mono::now()),
                _ => hprintln!("timeout"),
            }
        }
        // ANCHOR_END: timeout_at_basic

        debug::exit(debug::EXIT_SUCCESS);
    }
}

// Emulate some hal
async fn hal_get(n: u32) -> u32 {
    // emulate some delay time dependent on n
    let d = 350.millis() + n * 100.millis();
    hprintln!("the hal takes a duration of {:?}", d);
    Mono::delay(d).await;
    // emulate some return value
    5
}
