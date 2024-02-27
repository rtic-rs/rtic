//! examples/resource.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        // A resource
        shared: u32,
    }

    // Should not collide with the struct above
    #[allow(dead_code)]
    struct Shared2 {
        // A resource
        shared: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        rtic::pend(Interrupt::UART0);
        rtic::pend(Interrupt::UART1);

        (Shared { shared: 0 }, Local {})
    }

    // `shared` cannot be accessed from this context
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        // error: no `shared` field in `idle::Context`
        // _cx.shared.shared += 1;

        loop {}
    }

    // `shared` can be accessed from this context
    #[task(binds = UART0, shared = [shared])]
    fn uart0(mut cx: uart0::Context) {
        let shared = cx.shared.shared.lock(|shared| {
            *shared += 1;
            *shared
        });

        hprintln!("UART0: shared = {}", shared);
    }

    // `shared` can be accessed from this context
    #[task(binds = UART1, shared = [shared])]
    fn uart1(mut cx: uart1::Context) {
        let shared = cx.shared.shared.lock(|shared| {
            *shared += 1;
            *shared
        });

        hprintln!("UART1: shared = {}", shared);
    }
}
