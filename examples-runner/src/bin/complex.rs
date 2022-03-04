//! examples/complex.rs

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
    struct Shared {
        s2: u32, // shared with ceiling 2
        s3: u32, // shared with ceiling 3
        s4: u32, // shared with ceiling 4
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        println!("init");

        (
            Shared {
                s2: 0,
                s3: 0,
                s4: 0,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle(shared = [s2, s3])]
    fn idle(mut cx: idle::Context) -> ! {
        println!("idle p0 started");
        rtic::pend(Interrupt::GPIOC);
        cx.shared.s3.lock(|s| {
            println!("idle enter lock s3 {}", s);
            println!("idle pend t0");
            rtic::pend(Interrupt::GPIOA); // t0 p2, with shared ceiling 3
            println!("idle pend t1");
            rtic::pend(Interrupt::GPIOB); // t1 p3, with shared ceiling 3
            println!("idle pend t2");
            rtic::pend(Interrupt::GPIOC); // t2 p4, no sharing
            println!("idle still in lock s3 {}", s);
        });
        println!("\nback in idle");

        cx.shared.s2.lock(|s| {
            println!("enter lock s2 {}", s);
            println!("idle pend t0");
            rtic::pend(Interrupt::GPIOA); // t0 p2, with shared ceiling 2
            println!("idle pend t1");
            rtic::pend(Interrupt::GPIOB); // t1 p3, no sharing
            println!("idle pend t2");
            rtic::pend(Interrupt::GPIOC); // t2 p4, no sharing
            println!("idle still in lock s2 {}", s);
        });
        println!("\nidle exit");

        exit();

        // loop {
        //     cortex_m::asm::nop();
        // }
    }

    #[task(binds = GPIOA, priority = 2, local = [times: u32 = 0], shared = [s2, s3])]
    fn t0(cx: t0::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        println!(
            "t0 p2 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );
        println!("t0 p2 exit");
    }

    #[task(binds = GPIOB, priority = 3, local = [times: u32 = 0], shared = [s3, s4])]
    fn t1(mut cx: t1::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        println!(
            "t1 p3 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );

        cx.shared.s4.lock(|s| {
            println!("t1 enter lock s4 {}", s);
            println!("t1 pend t0");
            rtic::pend(Interrupt::GPIOA); // t0 p2, with shared ceiling 2
            println!("t1 pend t2");
            rtic::pend(Interrupt::GPIOC); // t2 p4, no sharing
            println!("t1 still in lock s4 {}", s);
        });

        println!("t1 p3 exit");
    }

    #[task(binds = GPIOC, priority = 4, local = [times: u32 = 0], shared = [s4])]
    fn t2(mut cx: t2::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        println!(
            "t2 p4 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );

        cx.shared.s4.lock(|s| {
            println!("enter lock s4 {}", s);
            *s += 1;
        });
        println!("t3 p4 exit");
    }
}
