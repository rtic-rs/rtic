//! examples/periodic-at2.rs

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

        let mut mono = Systick::new(systick, 1_000_000);

        foo::spawn_after(1.secs(), mono.now()).unwrap();

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    // Using the explicit type of the timer implementation
    #[task(local = [cnt: u32 = 0])]
    fn foo(cx: foo::Context, instant: fugit::TimerInstantU64<100>) {
        println!("foo ticks: {:?}", instant.ticks());
        *cx.local.cnt += 1;

        if *cx.local.cnt == 4 {
            exit();
        }

        // Spawn a new message with 100 ms offset to spawned time
        let next_instant = instant + 100.millis();
        bar::spawn_at(next_instant, next_instant).unwrap();
    }

    // Using the Instant from the Monotonic trait
    // This remains agnostic to the timer implementation
    #[task(local = [cnt: u32 = 0])]
    fn bar(_cx: bar::Context, instant: <MyMono as rtic_monotonic::Monotonic>::Instant) {
        println!("bar ticks: {:?}", instant.ticks());

        // Spawn a new message with 1s offset to spawned time
        let next_instant = instant + 1.secs();
        foo::spawn_at(next_instant, next_instant).unwrap();
    }
}
