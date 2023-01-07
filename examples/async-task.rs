#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

// NOTES:
//
// - Async tasks cannot have `#[lock_free]` resources, as they can interleave and each async
//   task can have a mutable reference stored.
// - Spawning an async task equates to it being polled once.

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        a: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        hprintln!("init").unwrap();

        async_task::spawn().unwrap();
        async_task2::spawn().unwrap();

        (Shared { a: 0 }, Local {})
    }

    #[idle(shared = [a])]
    fn idle(_: idle::Context) -> ! {
        // debug::exit(debug::EXIT_SUCCESS);
        loop {
            // hprintln!("idle");
            cortex_m::asm::wfi(); // put the MCU in sleep mode until interrupt occurs
        }
    }

    #[task(binds = UART1, shared = [a])]
    fn hw_task(cx: hw_task::Context) {
        let hw_task::SharedResources { a: _, .. } = cx.shared;
        hprintln!("hello from hw").ok();
    }

    #[task(shared = [a])]
    async fn async_task(cx: async_task::Context) {
        let async_task::SharedResources { a: _, .. } = cx.shared;
        hprintln!("hello from async").ok();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(priority = 2, shared = [a])]
    async fn async_task2(cx: async_task2::Context) {
        let async_task2::SharedResources { a: _, .. } = cx.shared;
        hprintln!("hello from async2").ok();
    }
}
