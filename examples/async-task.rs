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

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init").unwrap();

        async_task::spawn().unwrap();

        (Shared {}, Local {})
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
    async fn async_task(_cx: async_task::Context) {
        hprintln!("hello from async").ok();

        debug::exit(debug::EXIT_SUCCESS);
    }
}
