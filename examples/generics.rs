//! examples/generics.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtfm::{Exclusive, Mutex};

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(_: init::Context) {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);
    }

    #[task(binds = UART0, resources = [shared])]
    fn uart0(c: uart0::Context) {
        static mut STATE: u32 = 0;

        hprintln!("UART0(STATE = {})", *STATE).unwrap();

        // second argument has type `resources::shared`
        advance(STATE, c.resources.shared);

        rtfm::pend(Interrupt::UART1);

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = UART1, priority = 2, resources = [shared])]
    fn uart1(c: uart1::Context) {
        static mut STATE: u32 = 0;

        hprintln!("UART1(STATE = {})", *STATE).unwrap();

        // just to show that `shared` can be accessed directly
        *c.resources.shared += 0;

        // second argument has type `Exclusive<u32>`
        advance(STATE, Exclusive(c.resources.shared));
    }
};

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
