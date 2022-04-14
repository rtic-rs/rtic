//! examples/periodic.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [SSI0])]
mod app {
    use examples_runner::{println, exit};
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

        // Initialize the monotonic
        let mono = Systick::new(systick, 1_000_000);

        foo::spawn_after(1.secs()).unwrap();

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[task(local = [cnt: u32 = 0])]
    fn foo(cx: foo::Context) {
        println!("foo");
        *cx.local.cnt += 1;

        if *cx.local.cnt == 4 {
            exit();
        }

        // Periodic ever 1 seconds
        foo::spawn_after(1.secs()).unwrap();
    }
}
