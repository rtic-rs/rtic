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

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        local_to_uart0: i64,
        local_to_uart1: i64,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);
        rtic::pend(Interrupt::UART1);

        (
            Shared {},
            // initial values for the `#[local]` resources
            Local {
                local_to_uart0: 0,
                local_to_uart1: 0,
            },
            init::Monotonics(),
        )
    }

    // `#[local]` resources cannot be accessed from this context
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);

        // error: no `local` field in `idle::Context`
        // _cx.local.local_to_uart0 += 1;

        // error: no `local` field in `idle::Context`
        // _cx.local.local_to_uart1 += 1;

        loop {
            cortex_m::asm::nop();
        }
    }

    // `local_to_uart0` can only be accessed from this context
    // defaults to priority 1
    #[task(binds = UART0, local = [local_to_uart0])]
    fn uart0(cx: uart0::Context) {
        *cx.local.local_to_uart0 += 1;
        let local_to_uart0 = cx.local.local_to_uart0;

        // error: no `local_to_uart1` field in `uart0::LocalResources`
        cx.local.local_to_uart1 += 1;

        hprintln!("UART0: local_to_uart0 = {}", local_to_uart0).unwrap();
    }

    // `shared` can only be accessed from this context
    // explicitly set to priority 2
    #[task(binds = UART1, local = [local_to_uart1], priority = 2)]
    fn uart1(cx: uart1::Context) {
        *cx.local.local_to_uart1 += 1;
        let local_to_uart1 = cx.local.local_to_uart1;

        // error: no `local_to_uart0` field in `uart1::LocalResources`
        // cx.local.local_to_uart0 += 1;

        hprintln!("UART1: local_to_uart1 = {}", local_to_uart1).unwrap();
    }
}
