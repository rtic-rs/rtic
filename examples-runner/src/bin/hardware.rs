//! examples/hardware.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::pac::Interrupt;
    use examples_runner::{exit, println};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        // Pends the UART0 interrupt but its handler won't run until *after*
        // `init` returns because interrupts are disabled
        rtic::pend(Interrupt::UART0); // equivalent to NVIC::pend

        println!("init");

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // interrupts are enabled again; the `UART0` handler runs at this point

        println!("idle");

        rtic::pend(Interrupt::UART0);

        exit();

        // loop {
        //     cortex_m::asm::nop();
        // }
    }

    #[task(binds = UART0, local = [times: u32 = 0])]
    fn uart0(cx: uart0::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        println!(
            "UART0 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );
    }
}
