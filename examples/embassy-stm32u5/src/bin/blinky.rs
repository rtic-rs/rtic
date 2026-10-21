#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use embassy_stm32::exti;
use embassy_stm32::gpio;
use embassy_stm32::{bind_interrupts, interrupt};
use panic_probe as _;
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs{
        EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
});

systick_monotonic!(Mono, 1_000);

pub mod pac {
    pub use embassy_stm32::pac::Interrupt as interrupt;
    pub use embassy_stm32::pac::*;
}

// Peripherals shall be false when using with embassy-stm32 HAL
// SPI1 is just an unused interrupt needed by RTIC to dispatch all tasks
#[app(device = pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        blinking_period: u32,
    }

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

        let button = exti::ExtiInput::new(p.PC13, p.EXTI13, gpio::Pull::Down, Irqs);
        increase_blinking_period::spawn(button).ok();

        let mut led = gpio::Output::new(p.PB7, gpio::Level::High, gpio::Speed::Low);
        led.set_high();

        // Schedule the blinking task
        blink::spawn(led).ok();

        defmt::info!("Let's go!");
        (
            Shared {
                blinking_period: 500,
            },
            Local { counts: 0 },
        )
    }

    #[task(shared = [blinking_period], local = [counts], priority = 1)]
    async fn blink(mut cx: blink::Context, mut led: gpio::Output<'static>) {
        loop {
            led.toggle();
            *cx.local.counts += 1;
            defmt::info!("blink ({})", cx.local.counts);
            let period = cx.shared.blinking_period.lock(|val| *val);
            Mono::delay(period.millis()).await;
        }
    }

    #[task(shared = [blinking_period], priority = 1)]
    async fn increase_blinking_period(
        mut cx: increase_blinking_period::Context,
        mut button: exti::ExtiInput<'static, embassy_stm32::mode::Async>,
    ) {
        loop {
            button.wait_for_falling_edge().await;
            cx.shared.blinking_period.lock(|val| *val += 100);
        }
    }
}
