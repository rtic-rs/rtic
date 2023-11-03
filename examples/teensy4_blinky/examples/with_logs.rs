#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use bsp::pins::common::{P0, P1};
imxrt_uart_panic::register!(LPUART6, P1, P0, 115200, teensy4_panic::sos);

use teensy4_bsp as bsp;

use bsp::board;
use bsp::hal;
use bsp::logging;

use embedded_hal::serial::Write;

use rtic_monotonics::imxrt::Gpt1 as Mono;
use rtic_monotonics::imxrt::*;
use rtic_monotonics::Monotonic;

#[rtic::app(device = teensy4_bsp, dispatchers = [LPSPI1])]
mod app {
    use super::*;

    const LOG_POLL_INTERVAL: u32 = board::PERCLK_FREQUENCY / 100;
    const LOG_DMA_CHANNEL: usize = 0;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: board::Led,
        poll_log: hal::pit::Pit<3>,
        log_poller: logging::Poller,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut dma,
            pit: (_, _, _, mut poll_log),
            pins,
            lpuart6,
            mut gpio2,
            mut gpt1,
            ..
        } = board::t40(cx.device);

        // Logging
        let log_dma = dma[LOG_DMA_CHANNEL].take().unwrap();
        let mut log_uart = board::lpuart(lpuart6, pins.p1, pins.p0, 115200);
        for &ch in "\r\n===== Teensy4 Rtic Blinky =====\r\n\r\n".as_bytes() {
            nb::block!(log_uart.write(ch)).unwrap();
        }
        nb::block!(log_uart.flush()).unwrap();
        let log_poller =
            logging::log::lpuart(log_uart, log_dma, logging::Interrupts::Enabled).unwrap();
        poll_log.set_interrupt_enable(true);
        poll_log.set_load_timer_value(LOG_POLL_INTERVAL);
        poll_log.enable();

        // Initialize Monotonic
        gpt1.set_clock_source(hal::gpt::ClockSource::PeripheralClock);
        let gpt1_mono_token = rtic_monotonics::create_imxrt_gpt1_token!();
        Mono::start(board::PERCLK_FREQUENCY, gpt1.release(), gpt1_mono_token);

        // Setup LED
        let led = board::led(&mut gpio2, pins.p13);
        led.set();

        // Schedule the blinking task
        blink::spawn().ok();

        (
            Shared {},
            Local {
                log_poller,
                poll_log,
                led,
            },
        )
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let blink::LocalResources { led, .. } = cx.local;

        let mut next_update = Mono::now();

        loop {
            led.toggle();
            log::info!("Time: {}", Mono::now());
            next_update += 1000.millis();
            Mono::delay_until(next_update).await;
        }
    }

    #[task(binds = PIT, priority = 1, local = [poll_log, log_poller])]
    fn logger(cx: logger::Context) {
        let logger::LocalResources {
            poll_log,
            log_poller,
            ..
        } = cx.local;

        if poll_log.is_elapsed() {
            poll_log.clear_elapsed();

            log_poller.poll();
        }
    }
}
