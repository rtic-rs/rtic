//! examples/periodic.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic::time::duration::*;
    use systick_monotonic::Systick;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let systick = cx.core.SYST;

        let mono = Systick::new(systick, 12_000_000);

        foo::spawn_after(1.seconds()).unwrap();

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[task(local = [cnt: u32 = 0])]
    fn foo(cx: foo::Context) {
        hprintln!("foo").ok();
        *cx.local.cnt += 1;

        if *cx.local.cnt == 4 {
            debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        }

        // Periodic ever 1 seconds
        foo::spawn_after(1.seconds()).unwrap();
    }
}
