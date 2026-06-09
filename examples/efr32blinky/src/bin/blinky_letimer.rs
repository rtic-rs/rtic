//! Blinky on the **LETIMER** monotonic (32.768 kHz; runs in EM2+ deep sleep).
//!
//! Run: `cargo run --bin blinky_letimer` (see README for board selection).
#![no_main]
#![no_std]

use efr32blinky::Led;
use rtic_monotonics::silabs::letimer::prelude::*;
use {defmt_rtt as _, panic_probe as _};

// Default 32_768 Hz tick.
silabs_letimer_monotonic!(Mono);

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
        // Board-specific LF clock bring-up (LFRCO on brd2713a, LFXO on XIAO).
        efr32blinky::init_lf_clock();

        Mono::start(silabs_metapac::LETIMER0, &silabs_metapac::CMU);
        let led = Led::new();

        blink::spawn().ok();

        defmt::info!("init done (LETimer monotonic), starting blink");
        (Shared {}, Local { led })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let led = cx.local.led;
        loop {
            defmt::info!("blink @ {} ticks", Mono::now().ticks());
            led.set_high();
            Mono::delay(100.millis()).await;

            led.set_low();
            Mono::delay(900.millis()).await;
        }
    }
}
