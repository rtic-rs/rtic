//! examples/async-timeout.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;
use rtic_monotonics::systick::*;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use super::*;
    use futures::{future::FutureExt, select_biased};
    use rtic_monotonics::Monotonic;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init");

        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 12_000_000, systick_token);

        foo::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        // Call hal with short relative timeout using `select_biased`
        select_biased! {
            v = hal_get(1).fuse() => hprintln!("hal returned {}", v),
            _ = Systick::delay(200.millis()).fuse() =>  hprintln!("timeout", ), // this will finish first
        }

        // Call hal with long relative timeout using `select_biased`
        select_biased! {
            v = hal_get(1).fuse() => hprintln!("hal returned {}", v), // hal finish first
            _ = Systick::delay(1000.millis()).fuse() =>  hprintln!("timeout", ),
        }

        // Call hal with long relative timeout using monotonic `timeout_after`
        match Systick::timeout_after(1000.millis(), hal_get(1)).await {
            Ok(v) => hprintln!("hal returned {}", v),
            _ => hprintln!("timeout"),
        }

        // get the current time instance
        let mut instant = Systick::now();

        // do this 3 times
        for n in 0..3 {
            // absolute point in time without drift
            instant += 1000.millis();
            Systick::delay_until(instant).await;

            // absolute point it time for timeout
            let timeout = instant + 500.millis();
            hprintln!("now is {:?}, timeout at {:?}", Systick::now(), timeout);

            match Systick::timeout_at(timeout, hal_get(n)).await {
                Ok(v) => hprintln!("hal returned {} at time {:?}", v, Systick::now()),
                _ => hprintln!("timeout"),
            }
        }

        debug::exit(debug::EXIT_SUCCESS);
    }
}

// Emulate some hal
async fn hal_get(n: u32) -> u32 {
    // emulate some delay time dependent on n
    let d = 350.millis() + n * 100.millis();
    hprintln!("the hal takes a duration of {:?}", d);
    Systick::delay(d).await;
    // emulate some return value
    5
}
