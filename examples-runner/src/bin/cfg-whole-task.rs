//! examples/cfg-whole-task.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac,  dispatchers = [SSI0, QEI0])]
mod app {
    use examples_runner::exit;

    #[shared]
    struct Shared {
        count: u32,
        #[cfg(never)]
        unused: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();
        foo::spawn().unwrap();

        (
            Shared {
                count: 0,
                #[cfg(never)]
                unused: 1,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        exit();

        // loop {
        //     cortex_m::asm::nop();
        // }
    }

    #[task(capacity = 2, shared = [count])]
    fn foo(mut _cx: foo::Context) {
        #[cfg(debug_assertions)]
        {
            _cx.shared.count.lock(|count| *count += 1);

            log::spawn(_cx.shared.count.lock(|count| *count)).unwrap();
        }

        // this wouldn't compile in `release` mode
        // *_cx.shared.count += 1;

        // ..
    }

    // The whole task should disappear,
    // currently still present in the Tasks enum
    #[cfg(never)]
    #[task(capacity = 2, shared = [count])]
    fn foo2(mut _cx: foo2::Context) {
        #[cfg(debug_assertions)]
        {
            _cx.shared.count.lock(|count| *count += 10);

            log::spawn(_cx.shared.count.lock(|count| *count)).unwrap();
        }

        // this wouldn't compile in `release` mode
        // *_cx.shared.count += 1;

        // ..
    }

    #[cfg(debug_assertions)]
    #[task(capacity = 2)]
    fn log(_: log::Context, n: u32) {
        println!(
            "foo has been called {} time{}",
            n,
            if n == 1 { "" } else { "s" }
        );
    }
}
