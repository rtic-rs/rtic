#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use nrf52840_blinky::hal;

use rtic_monotonics::nrf::timer::prelude::*;
nrf_timer0_monotonic!(Mono, 8_000_000);

#[rtic::app(device = hal::pac, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use hal::gpio::{Level, Output, Pin, PushPull};
    use hal::prelude::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Initialize Monotonic
        Mono::start(cx.device.TIMER0);

        // Setup LED
        let port0 = hal::gpio::p0::Parts::new(cx.device.P0);
        let led = port0.p0_06.into_push_pull_output(Level::Low).degrade();

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { led })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let blink::LocalResources { led, .. } = cx.local;

        let mut next_tick = Mono::now();
        let mut blink_on = false;
        loop {
            let now = Mono::now();
            let now_ms: fugit::SecsDurationU64 = now.duration_since_epoch().convert();
            defmt::println!("Timer {} ({})", now_ms, now.ticks());

            blink_on = !blink_on;
            if blink_on {
                led.set_high().unwrap();
            } else {
                led.set_low().unwrap();
            }

            next_tick += 1000.millis();
            Mono::delay_until(next_tick).await;
        }
    }
}
