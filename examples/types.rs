//! examples/types.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
use rtfm::{app, Exclusive, Instant};

#[app(device = lm3s6965)]
const APP: () = {
    static mut SHARED: u32 = 0;

    #[init(schedule = [foo], spawn = [foo])]
    fn init() {
        let _: Instant = start;
        let _: rtfm::Peripherals = core;
        let _: lm3s6965::Peripherals = device;
        let _: init::Schedule = schedule;
        let _: init::Spawn = spawn;

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[exception(schedule = [foo], spawn = [foo])]
    fn SVCall() {
        let _: Instant = start;
        let _: SVCall::Schedule = schedule;
        let _: SVCall::Spawn = spawn;
    }

    #[interrupt(resources = [SHARED], schedule = [foo], spawn = [foo])]
    fn UART0() {
        let _: Instant = start;
        let _: resources::SHARED = resources.SHARED;
        let _: UART0::Schedule = schedule;
        let _: UART0::Spawn = spawn;
    }

    #[task(priority = 2, resources = [SHARED], schedule = [foo], spawn = [foo])]
    fn foo() {
        let _: Instant = scheduled;
        let _: Exclusive<u32> = resources.SHARED;
        let _: foo::Resources = resources;
        let _: foo::Schedule = schedule;
        let _: foo::Spawn = spawn;
    }

    extern "C" {
        fn UART1();
    }
};
