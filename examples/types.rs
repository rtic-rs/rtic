//! examples/types.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::debug;
use panic_semihosting as _;
use rtfm::cyccnt;

#[rtfm::app(device = lm3s6965, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init(schedule = [foo], spawn = [foo])]
    fn init(cx: init::Context) {
        let _: cyccnt::Instant = cx.start;
        let _: rtfm::Peripherals = cx.core;
        let _: lm3s6965::Peripherals = cx.device;
        let _: init::Schedule = cx.schedule;
        let _: init::Spawn = cx.spawn;

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[idle(schedule = [foo], spawn = [foo])]
    fn idle(cx: idle::Context) -> ! {
        let _: idle::Schedule = cx.schedule;
        let _: idle::Spawn = cx.spawn;

        loop {}
    }

    #[task(binds = UART0, resources = [shared], schedule = [foo], spawn = [foo])]
    fn uart0(cx: uart0::Context) {
        let _: cyccnt::Instant = cx.start;
        let _: resources::shared = cx.resources.shared;
        let _: uart0::Schedule = cx.schedule;
        let _: uart0::Spawn = cx.spawn;
    }

    #[task(priority = 2, resources = [shared], schedule = [foo], spawn = [foo])]
    fn foo(cx: foo::Context) {
        let _: cyccnt::Instant = cx.scheduled;
        let _: &mut u32 = cx.resources.shared;
        let _: foo::Resources = cx.resources;
        let _: foo::Schedule = cx.schedule;
        let _: foo::Spawn = cx.spawn;
    }

    extern "C" {
        fn UART1();
    }
};
