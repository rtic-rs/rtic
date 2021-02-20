//! examples/resource.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[resources]
    struct Resources {
        // A resource
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        rtic::pend(Interrupt::UART0);
        rtic::pend(Interrupt::UART1);

        (init::LateResources {}, init::Monotonics())
    }

    // `shared` cannot be accessed from this context
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);

        // error: no `resources` field in `idle::Context`
        // _cx.resources.shared += 1;

        loop {
            cortex_m::asm::nop();
        }
    }

    // `shared` can be accessed from this context
    // defaults to priority 1
    #[task(binds = UART0, resources = [shared])]
    fn uart0(mut cx: uart0::Context) {
        let shared = cx.resources.shared.lock(|shared| {
            *shared += 1;
            *shared
        });

        hprintln!("UART0: shared = {}", shared).unwrap();
    }

    // `shared` can be accessed from this context
    // explicitly set to priority 2
    #[task(binds = UART1, resources = [shared], priority = 2)]
    fn uart1(mut cx: uart1::Context) {
        let shared = cx.resources.shared.lock(|shared| {
            *shared += 1;
            *shared
        });

        hprintln!("UART1: shared = {}", shared).unwrap();
    }
}
