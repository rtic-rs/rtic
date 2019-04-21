//! examples/types.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
use rtfm::{Exclusive, Instant};

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    static mut SHARED: u32 = 0;

    #[init(schedule = [foo], spawn = [foo])]
    fn init(c: init::Context) {
        let _: Instant = c.start;
        let _: rtfm::Peripherals = c.core;
        let _: lm3s6965::Peripherals = c.device;
        let _: init::Schedule = c.schedule;
        let _: init::Spawn = c.spawn;

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[exception(schedule = [foo], spawn = [foo])]
    fn SVCall(c: SVCall::Context) {
        let _: Instant = c.start;
        let _: SVCall::Schedule = c.schedule;
        let _: SVCall::Spawn = c.spawn;
    }

    #[interrupt(resources = [SHARED], schedule = [foo], spawn = [foo])]
    fn UART0(c: UART0::Context) {
        let _: Instant = c.start;
        let _: resources::SHARED = c.resources.SHARED;
        let _: UART0::Schedule = c.schedule;
        let _: UART0::Spawn = c.spawn;
    }

    #[task(priority = 2, resources = [SHARED], schedule = [foo], spawn = [foo])]
    fn foo(c: foo::Context) {
        let _: Instant = c.scheduled;
        let _: Exclusive<u32> = c.resources.SHARED;
        let _: foo::Resources = c.resources;
        let _: foo::Schedule = c.schedule;
        let _: foo::Spawn = c.spawn;
    }

    extern "C" {
        fn UART1();
    }
};
