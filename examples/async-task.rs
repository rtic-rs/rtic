#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;
use systick_monotonic::*;

// NOTES:
//
// - Async tasks cannot have `#[lock_free]` resources, as they can interleve and each async
//   task can have a mutable reference stored.
// - Spawning an async task equates to it being polled at least once.
// - ...

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use crate::*;

    pub type AppInstant = <Systick<100> as rtic::Monotonic>::Instant;
    pub type AppDuration = <Systick<100> as rtic::Monotonic>::Duration;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("init").unwrap();

        normal_task::spawn().ok();
        async_task::spawn().ok();

        (
            Shared {},
            Local {},
            init::Monotonics(Systick::new(cx.core.SYST, 12_000_000)),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // debug::exit(debug::EXIT_SUCCESS);
        loop {
            // hprintln!("idle");
            cortex_m::asm::wfi(); // put the MCU in sleep mode until interrupt occurs
        }
    }

    #[task]
    fn normal_task(_cx: normal_task::Context) {
        hprintln!("hello from normal").ok();
    }

    #[task]
    async fn async_task(_cx: async_task::Context) {
        hprintln!("hello from async").ok();

        debug::exit(debug::EXIT_SUCCESS);
    }
}
