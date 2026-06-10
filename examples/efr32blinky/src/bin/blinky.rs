//! Blinky on a **TIMER** monotonic (high-frequency, 1 MHz tick). The instance
//! is picked by the `timerN` feature (see README / Cargo.toml).
//!
//! Run: `cargo run --bin blinky` (see README for board/timer selection).
#![no_main]
#![no_std]

use efr32blinky::{Led, TIMER0_CLOCK_HZ};
use rtic_monotonics::silabs::timer::prelude::*;
use {defmt_rtt as _, panic_probe as _};

// The TIMER instance is selected by the `timerN` feature.
#[cfg(feature = "timer0")]
silabs_timer0_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer1")]
silabs_timer1_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer2")]
silabs_timer2_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer3")]
silabs_timer3_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer4")]
silabs_timer4_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer5")]
silabs_timer5_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer6")]
silabs_timer6_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer7")]
silabs_timer7_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer8")]
silabs_timer8_monotonic!(Mono, 1_000_000);
#[cfg(feature = "timer9")]
silabs_timer9_monotonic!(Mono, 1_000_000);
#[cfg(not(any(
    feature = "timer0",
    feature = "timer1",
    feature = "timer2",
    feature = "timer3",
    feature = "timer4",
    feature = "timer5",
    feature = "timer6",
    feature = "timer7",
    feature = "timer8",
    feature = "timer9",
)))]
compile_error!("enable one TIMER feature, e.g. `timer0` (see Cargo.toml)");

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

        defmt::info!("init done (TIMER monotonic), starting blink");
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
