//! examples/complex.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {

    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        s2: u32, // shared with ceiling 2
        s3: u32, // shared with ceiling 3
        s4: u32, // shared with ceiling 4
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        hprintln!("init");

        (
            Shared {
                s2: 0,
                s3: 0,
                s4: 0,
            },
            Local {},
        )
    }

    #[idle(shared = [s2, s3])]
    fn idle(mut cx: idle::Context) -> ! {
        hprintln!("idle p0 started");
        rtic::pend(Interrupt::GPIOC);
        cx.shared.s3.lock(|s| {
            hprintln!("idle enter lock s3 {}", s);
            hprintln!("idle pend t0");
            rtic::pend(Interrupt::GPIOA); // t0 p2, with shared ceiling 3
            hprintln!("idle pend t1");
            rtic::pend(Interrupt::GPIOB); // t1 p3, with shared ceiling 3
            hprintln!("idle pend t2");
            rtic::pend(Interrupt::GPIOC); // t2 p4, no sharing
            hprintln!("idle still in lock s3 {}", s);
        });
        hprintln!("\nback in idle");

        cx.shared.s2.lock(|s| {
            hprintln!("enter lock s2 {}", s);
            hprintln!("idle pend t0");
            rtic::pend(Interrupt::GPIOA); // t0 p2, with shared ceiling 2
            hprintln!("idle pend t1");
            rtic::pend(Interrupt::GPIOB); // t1 p3, no sharing
            hprintln!("idle pend t2");
            rtic::pend(Interrupt::GPIOC); // t2 p4, no sharing
            hprintln!("idle still in lock s2 {}", s);
        });
        hprintln!("\nidle exit");

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = GPIOA, priority = 2, local = [times: u32 = 0], shared = [s2, s3])]
    fn t0(cx: t0::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        hprintln!(
            "t0 p2 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );
        hprintln!("t0 p2 exit");
    }

    #[task(binds = GPIOB, priority = 3, local = [times: u32 = 0], shared = [s3, s4])]
    fn t1(mut cx: t1::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        hprintln!(
            "t1 p3 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );

        cx.shared.s4.lock(|s| {
            hprintln!("t1 enter lock s4 {}", s);
            hprintln!("t1 pend t0");
            rtic::pend(Interrupt::GPIOA); // t0 p2, with shared ceiling 2
            hprintln!("t1 pend t2");
            rtic::pend(Interrupt::GPIOC); // t2 p4, no sharing
            hprintln!("t1 still in lock s4 {}", s);
        });

        hprintln!("t1 p3 exit");
    }

    #[task(binds = GPIOC, priority = 4, local = [times: u32 = 0], shared = [s4])]
    fn t2(mut cx: t2::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        hprintln!(
            "t2 p4 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );

        cx.shared.s4.lock(|s| {
            hprintln!("enter lock s4 {}", s);
            *s += 1;
        });
        hprintln!("t3 p4 exit");
    }
}
