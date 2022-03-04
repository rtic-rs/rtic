//! examples/extern_binds.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;
use examples_runner::println;

// Free function implementing the interrupt bound task `foo`.
fn foo(_: app::foo::Context) {
    println!("foo called");
}

#[rtic::app(device = examples_runner::pac)]
mod app {
    use crate::foo;
    use examples_runner::pac::Interrupt;
    use examples_runner::{exit, println};

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

        // loop {
        //     cortex_m::asm::nop();
        // }
    }

    extern "Rust" {
        #[task(binds = UART0)]
        fn foo(_: foo::Context);
    }
}
