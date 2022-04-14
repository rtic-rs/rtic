//! examples/binds.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

// `examples/interrupt.rs` rewritten to use `binds`
#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::{println, exit};
    use examples_runner::pac::Interrupt;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        println!("init");

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        println!("idle");

        rtic::pend(Interrupt::UART0);

        exit();
    }

    #[task(binds = UART0, local = [times: u32 = 0])]
    fn foo(cx: foo::Context) {
        *cx.local.times += 1;

        println!(
            "foo called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );
    }
}
