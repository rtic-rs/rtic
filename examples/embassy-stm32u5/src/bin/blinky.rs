#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use embassy_stm32::gpio::{Level, Output, Speed};
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use {defmt_rtt as _, panic_probe as _};

systick_monotonic!(Mono, 1_000);

pub mod pac {
    pub use embassy_stm32::pac::Interrupt as interrupt;
    pub use embassy_stm32::pac::*;
}

#[app(device = pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        counts: u32,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut config = embassy_stm32::Config::default();
        config.rcc.ahb_pre = embassy_stm32::rcc::AHBPrescaler::DIV4;
        let p = embassy_stm32::init(config); // configure clocks FIRST

        // MSIS (4MHz) -> AHBP (DIV4)
        Mono::start(cx.core.SYST, 1_000_000);

        let mut led = Output::new(p.PB7, Level::High, Speed::Low);
        led.set_high();

        // Schedule the blinking task
        blink::spawn(led).ok();

        defmt::info!("Let's go!");
        (Shared {}, Local { counts: 0 })
    }

    #[task(local = [counts], priority = 1)]
    async fn blink(cx: blink::Context, mut led: Output<'static>) {
        loop {
            led.toggle();
            *cx.local.counts += 1;
            defmt::info!("blink ({})", *cx.local.counts);
            Mono::delay(1000.millis()).await;
        }
    }
}
