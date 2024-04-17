#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

#[panic_handler]
fn panic(_: &::core::panic::PanicInfo) -> ! {
    ::teensy4_panic::sos()
}

use teensy4_bsp::{board, hal};

use rtic_monotonics::imxrt::prelude::*;
imxrt_gpt1_monotonic!(Mono, board::PERCLK_FREQUENCY);

#[rtic::app(device = teensy4_bsp, dispatchers = [LPSPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: board::Led,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            pins,
            mut gpio2,
            mut gpt1,
            ..
        } = board::t40(cx.device);

        // Initialize Monotonic
        gpt1.set_clock_source(hal::gpt::ClockSource::PeripheralClock);
        Mono::start(gpt1.release());

        // Setup LED
        let led = board::led(&mut gpio2, pins.p13);
        led.set();

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { led })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let blink::LocalResources { led, .. } = cx.local;

        loop {
            led.toggle();
            Mono::delay(1000.millis()).await;
        }
    }
}
