//! [compile-pass] check that `#[cfg]` attributes are respected

#![no_main]
#![no_std]

use panic_halt as _;
use cortex_m::peripheral::DWT;
use rtic::time::{self, instant::Instant};

// NOTE: does NOT properly work on QEMU
#[rtic::app(device = lm3s6965, monotonic = crate::CYCCNT, sys_timer_freq = 64_000_000)]
const APP: () = {
    struct Resources {
        #[cfg(never)]
        #[init(0)]
        foo: u32,
    }

    #[init]
    fn init(_: init::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        #[cfg(never)]
        static mut BAR: u32 = 0;

        loop {}
    }

    #[task(resources = [foo], schedule = [quux], spawn = [quux])]
    fn foo(_: foo::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[task(priority = 3, resources = [foo], schedule = [quux], spawn = [quux])]
    fn bar(_: bar::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[cfg(never)]
    #[task]
    fn quux(_: quux::Context) {}

    extern "C" {
        fn UART0();
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
    const PERIOD: time::Period = time::Period::new_raw(1, 64_000_000);

    fn now() -> Instant<Self> {
        let ticks = DWT::get_cycle_count();

        Instant::new(ticks as i32)
    }
}
