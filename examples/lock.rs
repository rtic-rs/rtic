//! examples/lock.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use rtfm::app;

macro_rules! println {
    ($($tt:tt)*) => {
        if let Ok(mut stdout) = cortex_m_semihosting::hio::hstdout() {
            use core::fmt::Write;

            writeln!(stdout, $($tt)*).ok();
        }
    };
}

#[app(device = lm3s6965)]
const APP: () = {
    static mut SHARED: u32 = 0;

    #[init]
    fn init() {
        rtfm::pend(Interrupt::GPIOA);
    }

    // when omitted priority is assumed to be `1`
    #[interrupt(resources = [SHARED])]
    fn GPIOA() {
        println!("A");

        // the lower priority task requires a critical section to access the data
        resources.SHARED.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // GPIOB will *not* run right now due to the critical section
            rtfm::pend(Interrupt::GPIOB);

            println!("B - SHARED = {}", *shared);

            // GPIOC does not contend for `SHARED` so it's allowed to run now
            rtfm::pend(Interrupt::GPIOC);
        });

        // critical section is over: GPIOB can now start

        println!("E");

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[interrupt(priority = 2, resources = [SHARED])]
    fn GPIOB() {
        // the higher priority task does *not* need a critical section
        *resources.SHARED += 1;

        println!("D - SHARED = {}", *resources.SHARED);
    }

    #[interrupt(priority = 3)]
    fn GPIOC() {
        println!("C");
    }
};
