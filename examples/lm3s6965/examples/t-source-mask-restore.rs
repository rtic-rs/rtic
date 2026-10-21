//! examples/t-source-mask-restore.rs
//!
//! Checks that leaving a lock only re-enables the interrupts the lock masked. An interrupt the
//! task masks itself while holding the lock must stay masked afterwards.

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use cortex_m::peripheral::NVIC;
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        shared: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        // UART1 is not used by RTIC, but shares the NVIC enable register with the dispatchers
        // SAFETY: UART1 is never pended.
        unsafe { NVIC::unmask(Interrupt::UART1) };

        foo::spawn().unwrap();

        (Shared { shared: 0 }, Local {})
    }

    #[task(priority = 1, shared = [shared])]
    async fn foo(mut c: foo::Context) {
        hprintln!("UART1 enabled before lock: {}", NVIC::is_enabled(Interrupt::UART1));

        c.shared.shared.lock(|shared| {
            *shared += 1;
            NVIC::mask(Interrupt::UART1);
        });

        hprintln!("UART1 enabled after lock: {}", NVIC::is_enabled(Interrupt::UART1));

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    // Raises the ceiling of `shared` above `foo`, so `foo` has to lock it
    #[task(priority = 2, shared = [shared])]
    async fn bar(mut c: bar::Context) {
        c.shared.shared.lock(|shared| *shared += 1);
    }
}
