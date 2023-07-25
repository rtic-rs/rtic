//! examples/async-task-multiple-prios.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

// NOTES:
//
// - Async tasks cannot have `#[lock_free]` resources, as they can interleave and each async
//   task can have a mutable reference stored.
// - Spawning an async task equates to it being polled once.

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        a: u32,
        b: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        hprintln!("init");

        async_task1::spawn(1).ok();
        async_task2::spawn().ok();
        async_task3::spawn().ok();
        async_task4::spawn().ok();

        (Shared { a: 0, b: 0 }, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            hprintln!("idle");
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[task(priority = 1, shared = [a, b])]
    async fn async_task1(mut cx: async_task1::Context, inc: u32) {
        hprintln!(
            "hello from async 1 a {}",
            cx.shared.a.lock(|a| {
                *a += inc;
                *a
            })
        );
    }

    #[task(priority = 1, shared = [a, b])]
    async fn async_task2(mut cx: async_task2::Context) {
        hprintln!(
            "hello from async 2 a {}",
            cx.shared.a.lock(|a| {
                *a += 1;
                *a
            })
        );
    }

    #[task(priority = 2, shared = [a, b])]
    async fn async_task3(mut cx: async_task3::Context) {
        hprintln!(
            "hello from async 3 a {}",
            cx.shared.a.lock(|a| {
                *a += 1;
                *a
            })
        );
    }

    #[task(priority = 2, shared = [a, b])]
    async fn async_task4(mut cx: async_task4::Context) {
        hprintln!(
            "hello from async 4 a {}",
            cx.shared.a.lock(|a| {
                *a += 1;
                *a
            })
        );
    }
}
