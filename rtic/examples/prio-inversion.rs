//! examples/prio-inversion.rs
//!
//! Here we test to make sure we don't have priority inversion.

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;
use rtic::app;

// t1 p1 use b, a
// t2 p2 use a
// t3 p3
// t4 p4 use b
//
// so t1 start , take b take a, pend t3
// t3 should not start
// try to see if it starts, IT SHOULD NOT

#[app(device = lm3s6965, dispatchers = [SSI0, QEI0, GPIOA, GPIOB])]
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
        foo::spawn().unwrap();

        (Shared { a: 0, b: 0 }, Local {})
    }

    #[task(priority = 1, shared = [a, b])]
    async fn foo(cx: foo::Context) {
        let foo::SharedResources { mut a, mut b, .. } = cx.shared;

        hprintln!("foo - start");

        // basepri = 0
        b.lock(|b| {
            // basepri = max(basepri = 0, ceil(b) = 4) = 4
            a.lock(|a| {
                // basepri = max(basepri = 4, ceil(a) = 2) = 4

                hprintln!("pre baz spawn {} {}", a, b);

                // This spawn should be blocked as prio(baz) = 3
                baz::spawn().unwrap();

                hprintln!("post baz spawn {} {}", a, b);
            });
            // basepri = 4
        });
        // basepri = 0

        hprintln!("foo - end");
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2, shared = [a])]
    async fn bar(_: bar::Context) {
        hprintln!(" bar");
    }

    #[task(priority = 3)]
    async fn baz(_: baz::Context) {
        hprintln!(" baz - start");
        hprintln!(" baz - end");
    }

    #[task(priority = 4, shared = [b])]
    async fn pow(_: pow::Context) {
        hprintln!(" pow - start");
        hprintln!(" pow - end");
    }
}
