//! Blinky on the **TIMER0** monotonic (high-frequency, 1 MHz tick).
//!
//! Run: `cargo run --bin blinky` (see README for board selection).
#![no_main]
#![no_std]

use efr32blinky::{Led, TIMER0_CLOCK_HZ};
use rtic_monotonics::silabs::timer::prelude::*;
use {defmt_rtt as _, panic_probe as _};

silabs_timer0_monotonic!(Mono, 1_000_000);

// silabs-metapac works directly as the RTIC `device`; it owns no peripherals.
#[rtic::app(device = silabs_metapac, peripherals = false, dispatchers = [SW0])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
    }

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        Mono::start(TIMER0_CLOCK_HZ);
        let led = Led::new();

        blink::spawn().ok();

        defmt::info!("init done (TIMER0 monotonic), starting blink");
        (Shared {}, Local { led })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let led = cx.local.led;
        loop {
            defmt::info!("blink @ {} us", Mono::now().ticks());
            led.set_high();
            Mono::delay(100.millis()).await;

            led.set_low();
            Mono::delay(900.millis()).await;
        }
    }
}
