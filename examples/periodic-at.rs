//! examples/periodic-at.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use systick_monotonic::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let systick = cx.core.SYST;

        // Initialize the monotonic (SysTick rate in QEMU is 12 MHz)
        let mut mono = Systick::new(systick, 12_000_000);

        foo::spawn_after(1.secs(), mono.now()).unwrap();

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[task(local = [cnt: u32 = 0])]
    fn foo(cx: foo::Context, instant: fugit::TimerInstantU64<100>) {
        hprintln!("foo {:?}", instant).ok();
        *cx.local.cnt += 1;

        if *cx.local.cnt == 4 {
            debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        }

        // Periodic ever 1 seconds
        let next_instant = instant + 1.secs();
        foo::spawn_at(next_instant, next_instant).unwrap();
    }
}
