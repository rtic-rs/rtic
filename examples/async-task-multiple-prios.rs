#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

// NOTES:
//
// - Async tasks cannot have `#[lock_free]` resources, as they can interleve and each async
//   task can have a mutable reference stored.
// - Spawning an async task equates to it being polled once.

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use systick_monotonic::*;

    #[shared]
    struct Shared {
        a: u32,
        b: u32,
    }

    #[local]
    struct Local {}

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("init").unwrap();

        normal_task::spawn().ok();
        async_task::spawn().ok();
        normal_task2::spawn().ok();
        async_task2::spawn().ok();

        (
            Shared { a: 0, b: 0 },
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

    #[task(priority = 1, shared = [a, b])]
    fn normal_task(_cx: normal_task::Context) {
        hprintln!("hello from normal 1").ok();
    }

    #[task(priority = 1, shared = [a, b])]
    async fn async_task(_cx: async_task::Context) {
        hprintln!("hello from async 1").ok();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(priority = 2, shared = [a, b])]
    fn normal_task2(_cx: normal_task2::Context) {
        hprintln!("hello from normal 2").ok();
    }

    #[task(priority = 2, shared = [a, b])]
    async fn async_task2(_cx: async_task2::Context) {
        hprintln!("hello from async 2").ok();
    }
}
