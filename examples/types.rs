//! examples/types.rs

#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use cortex_m_semihosting::debug;
use panic_semihosting as _;
use rtic::time::{self, Instant};

// NOTE: does NOT properly work on QEMU
#[rtic::app(device = lm3s6965, peripherals = true, monotonic = crate::CYCCNT, sys_timer_freq = 64_000_000)]
const APP: () = {
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init(schedule = [foo], spawn = [foo])]
    fn init(cx: init::Context) {
        let _: Instant<CYCCNT> = cx.start;
        let _: rtic::Peripherals = cx.core;
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
        let _: Instant<CYCCNT> = cx.start;
        let _: resources::shared = cx.resources.shared;
        let _: uart0::Schedule = cx.schedule;
        let _: uart0::Spawn = cx.spawn;
    }

    #[task(priority = 2, resources = [shared], schedule = [foo], spawn = [foo])]
    fn foo(cx: foo::Context) {
        let _: Instant<CYCCNT> = cx.scheduled;
        let _: &mut u32 = cx.resources.shared;
        let _: foo::Resources = cx.resources;
        let _: foo::Schedule = cx.schedule;
        let _: foo::Spawn = cx.spawn;
    }

    extern "C" {
        fn UART1();
    }
};

/// Implementation of the `Monotonic` trait based on CYCle CouNTer
#[derive(Debug)]
pub struct CYCCNT;

impl rtic::Monotonic for CYCCNT {
    unsafe fn reset() {
        (0xE0001004 as *mut u32).write_volatile(0)
    }
}

impl time::Clock for CYCCNT {
    type Rep = i32;

    // the period of 64 MHz
    const PERIOD: time::Period = time::Period::new(1, 64_000_000);

    fn now() -> Instant<Self> {
        let ticks = DWT::get_cycle_count();

        Instant::new(ticks as i32)
    }
}
