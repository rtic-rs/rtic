//! examples/resource.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::{println, exit};
    use examples_runner::pac::Interrupt;

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
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);
        rtic::pend(Interrupt::UART1);

        (Shared { shared: 0 }, Local {}, init::Monotonics())
    }

    // `shared` cannot be accessed from this context
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        exit();

        // error: no `shared` field in `idle::Context`
        // _cx.shared.shared += 1;

        // loop {}
    }

    // `shared` can be accessed from this context
    #[task(binds = UART0, shared = [shared])]
    fn uart0(mut cx: uart0::Context) {
        let shared = cx.shared.shared.lock(|shared| {
            *shared += 1;
            *shared
        });

        println!("UART0: shared = {}", shared);
    }

    // `shared` can be accessed from this context
    #[task(binds = UART1, shared = [shared])]
    fn uart1(mut cx: uart1::Context) {
        let shared = cx.shared.shared.lock(|shared| {
            *shared += 1;
            *shared
        });

        println!("UART1: shared = {}", shared);
    }
}
