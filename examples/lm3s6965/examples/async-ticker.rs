//! examples/async-ticker.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0], peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_monotonics::systick::prelude::*;

    systick_monotonic!(Mono, 100);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init");

        Mono::start(cx.core.SYST, 12_000_000);

        foo::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        let start = Mono::now();
        let mut ticker = Mono::ticker_at(start, 100.millis());

        for _ in 0..4 {
            let at = ticker.tick().await;
            hprintln!("tick at {} ms", (at - start).as_millis());

            // Work between ticks does not move the following ticks.
            Mono::delay(30.millis()).await;
        }

        debug::exit(debug::EXIT_SUCCESS);
    }
}
