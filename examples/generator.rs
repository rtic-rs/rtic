//! examples/hardware.rs

#![feature(generator_trait)]
#![feature(generators)]
#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(0)]
        x: i64,
    }

    #[init]
    fn init(_: init::Context) {
        hprintln!("init").ok();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle").ok();

        rtfm::pend(Interrupt::UART0);
        hprintln!("C").ok();
        rtfm::pend(Interrupt::UART0);
        hprintln!("E").ok();
        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }

    #[task(binds = UART0, priority = 1, resources = [x])]
    fn uart0(mut cx: uart0::Context) -> impl Generator<Yield = (), Return = !> {
        hprintln!("A").ok();

        move || loop {
            hprintln!("B").ok();
            yield;

            cx.resources.x.lock(|x| {
                hprintln!("lock").ok();
                *x += 1;
            });

            hprintln!("D").ok();
            yield;
        }
    }

    #[task(binds = UART1, priority = 2, resources = [x])]
    fn uart1(_: uart1::Context) {}
};
