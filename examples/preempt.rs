//! examples/preempt.rs

#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        rtfm::pend(Interrupt::GPIOA);
    }

    #[task(binds = GPIOA, priority = 1)]
    fn gpioa(_: gpioa::Context) {
        hprintln!("GPIOA - start").unwrap();
        rtfm::pend(Interrupt::GPIOC);
        hprintln!("GPIOA - end").unwrap();
        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = GPIOB, priority = 2)]
    fn gpiob(_: gpiob::Context) {
        hprintln!(" GPIOB").unwrap();
    }

    #[task(binds = GPIOC, priority = 2)]
    fn gpioc(_: gpioc::Context) {
        hprintln!(" GPIOC - start").unwrap();
        rtfm::pend(Interrupt::GPIOB);
        hprintln!(" GPIOC - end").unwrap();
    }
};
