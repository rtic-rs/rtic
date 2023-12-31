//! examples/locals.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        local_to_foo: i64,
        local_to_bar: i64,
        local_to_idle: i64,
    }

    // `#[init]` cannot access locals from the `#[local]` struct as they are initialized here.
    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();
        bar::spawn().unwrap();

        (
            Shared {},
            // initial values for the `#[local]` resources
            Local {
                local_to_foo: 0,
                local_to_bar: 0,
                local_to_idle: 0,
            },
        )
    }

    // `local_to_idle` can only be accessed from this context
    #[idle(local = [local_to_idle])]
    fn idle(cx: idle::Context) -> ! {
        let local_to_idle = cx.local.local_to_idle;
        *local_to_idle += 1;

        hprintln!("idle: local_to_idle = {}", local_to_idle);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        // error: no `local_to_foo` field in `idle::LocalResources`
        // _cx.local.local_to_foo += 1;

        // error: no `local_to_bar` field in `idle::LocalResources`
        // _cx.local.local_to_bar += 1;

        loop {
            cortex_m::asm::nop();
        }
    }

    // `local_to_foo` can only be accessed from this context
    #[task(local = [local_to_foo], priority = 1)]
    async fn foo(cx: foo::Context) {
        let local_to_foo = cx.local.local_to_foo;
        *local_to_foo += 1;

        // error: no `local_to_bar` field in `foo::LocalResources`
        // cx.local.local_to_bar += 1;

        hprintln!("foo: local_to_foo = {}", local_to_foo);
    }

    // `local_to_bar` can only be accessed from this context
    #[task(local = [local_to_bar], priority = 1)]
    async fn bar(cx: bar::Context) {
        let local_to_bar = cx.local.local_to_bar;
        *local_to_bar += 1;

        // error: no `local_to_foo` field in `bar::LocalResources`
        // cx.local.local_to_foo += 1;

        hprintln!("bar: local_to_bar = {}", local_to_bar);
    }
}
