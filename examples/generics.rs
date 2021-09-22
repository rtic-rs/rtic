//! examples/generics.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rtic::Mutex;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
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

    #[task(binds = UART0, shared = [shared], local = [state: u32 = 0])]
    fn uart0(c: uart0::Context) {
        hprintln!("UART0(STATE = {})", *c.local.state).unwrap();

        // second argument has type `shared::shared`
        super::advance(c.local.state, c.shared.shared);

        rtic::pend(Interrupt::UART1);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(binds = UART1, priority = 2, shared = [shared], local = [state: u32 = 0])]
    fn uart1(c: uart1::Context) {
        hprintln!("UART1(STATE = {})", *c.local.state).unwrap();

        // second argument has type `shared::shared`
        super::advance(c.local.state, c.shared.shared);
    }
}

// the second parameter is generic: it can be any type that implements the `Mutex` trait
fn advance(state: &mut u32, mut shared: impl Mutex<T = u32>) {
    *state += 1;

    let (old, new) = shared.lock(|shared: &mut u32| {
        let old = *shared;
        *shared += *state;
        (old, *shared)
    });

    hprintln!("shared: {} -> {}", old, new).unwrap();
}
