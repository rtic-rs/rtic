//! examples/static-exec.rs
//!
//! Each task's executor lives in a `static`, so its future is in `.bss` rather than on `main`'s
//! stack frame.
//!
//! Covers a task with no arguments, one with several, two priorities so more than one dispatcher
//! polls a static, and the three ways a task's future gets built: inline, `extern` and diverging,
//! `extern` and borrowing its context.

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

/// An `extern` task that borrows its context, so its future is not `'static`.
async fn borrows_ctx(_c: app::borrows_ctx::Context<'_>) {
    hprintln!("borrows_ctx");
}

/// An `extern` task that never returns.
async fn diverges(_c: app::diverges::Context<'static>) -> ! {
    hprintln!("diverges");
    loop {
        core::future::pending::<()>().await;
    }
}

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0])]
mod app {
    use crate::{borrows_ctx, diverges};
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        plain::spawn().ok();
        borrows_ctx::spawn().ok();
        diverges::spawn().ok();
        with_args::spawn(1, 2, 3).ok();
        higher::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn plain(_: plain::Context) {
        hprintln!("plain");
    }

    // Spawned last on this dispatcher, so it is the one that ends the example: everything at this
    // priority has been polled by the time it runs, and the higher-priority task preempted first.
    #[task]
    async fn with_args(_: with_args::Context, a: u32, b: u16, c: u8) {
        hprintln!("with_args {} {} {}", a, b, c);
        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(priority = 2)]
    async fn higher(_: higher::Context) {
        hprintln!("higher");
    }

    extern "Rust" {
        #[task]
        async fn borrows_ctx(_: borrows_ctx::Context);

        #[task]
        async fn diverges(_: diverges::Context) -> !;
    }
}
