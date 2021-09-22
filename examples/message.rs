//! examples/message.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn(/* no message */).unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(local = [count: u32 = 0])]
    fn foo(cx: foo::Context) {
        hprintln!("foo").unwrap();

        bar::spawn(*cx.local.count).unwrap();
        *cx.local.count += 1;
    }

    #[task]
    fn bar(_: bar::Context, x: u32) {
        hprintln!("bar({})", x).unwrap();

        baz::spawn(x + 1, x + 2).unwrap();
    }

    #[task]
    fn baz(_: baz::Context, x: u32, y: u32) {
        hprintln!("baz({}, {})", x, y).unwrap();

        if x + y > 4 {
            debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        }

        foo::spawn().unwrap();
    }
}
