#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use defmt_rtt as _; // global logger

pub use stm32g0xx_hal as hal; // memory layout

use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

use rtic_monotonics::stm32::prelude::*;
stm32_tim3_monotonic!(Mono, 1_000_000);

#[rtic::app(device = hal::stm32, peripherals = true, dispatchers = [USART1, USART2])]
mod app {
    use super::*;

    #[local]
    struct LocalResources {}

    #[shared]
    struct SharedResources {}

    #[init]
    fn init(ctx: init::Context) -> (SharedResources, LocalResources) {
        // enable dma clock during sleep, otherwise defmt doesn't work
        ctx.device.RCC.ahbenr.modify(|_, w| w.dmaen().set_bit());

        defmt::println!("TIM Monotonic blinker example!");

        // Start the monotonic
        Mono::start(16_000_000);

        print_messages::spawn().unwrap();

        (SharedResources {}, LocalResources {})
    }

    #[task(priority = 2)]
    async fn print_messages(_cx: print_messages::Context) {
        let mut next_update = <Mono as Monotonic>::Instant::from_ticks(0u64);

        loop {
            defmt::println!("Time: {} ticks", Mono::now().ticks());
            next_update += 1000u64.millis();
            Mono::delay_until(next_update).await;
        }
    }
}
