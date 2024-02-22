#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_stm32::gpio::{Level, Output, Speed};
use rtic::app;
use rtic_monotonics::systick::*;
use {defmt_rtt as _, panic_probe as _};

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
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 25_000_000, systick_mono_token);

        let p = embassy_stm32::init(Default::default());
        info!("Hello World!");

        let mut led = Output::new(p.PC6, Level::High, Speed::Low);
        info!("high");
        led.set_high();

        // Schedule the blinking task
        blink::spawn(led).ok();

        (Shared {}, Local {})
    }

    #[task()]
    async fn blink(_cx: blink::Context, mut led: Output<'static, embassy_stm32::peripherals::PC6>) {
        let mut state = true;
        loop {
            info!("blink");
            if state {
                led.set_high();
            } else {
                led.set_low();
            }
            state = !state;
            Systick::delay(1000.millis()).await;
        }
    }
}
