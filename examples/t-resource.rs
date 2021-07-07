//! [compile-pass] Check code generation of shared resources

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[shared]
    struct Shared {
        o2: u32, // idle
        o3: u32, // EXTI0
        o4: u32, // idle
        o5: u32, // EXTI1
        s1: u32, // idle & uart0
        s2: u32, // uart0 & uart1
        s3: u32, // idle & uart0
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        (
            Shared {
                o2: 0,
                o3: 0,
                o4: 0,
                o5: 0,
                s1: 0,
                s2: 0,
                s3: 0,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle(shared = [o2, &o4, s1, &s3])]
    fn idle(mut c: idle::Context) -> ! {
        // owned by `idle` == `&'static mut`
        let _: shared_resources::o2 = c.shared.o2;

        // owned by `idle` == `&'static` if read-only
        let _: &u32 = c.shared.o4;

        // shared with `idle` == `Mutex`
        c.shared.s1.lock(|_| {});

        // `&` if read-only
        let _: &u32 = c.shared.s3;

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = UART0, shared = [o3, s1, s2, &s3])]
    fn uart0(c: uart0::Context) {
        // owned by interrupt == `&mut`
        let _: shared_resources::o3 = c.shared.o3;

        // no `Mutex` proxy when access from highest priority task
        let _: shared_resources::s1 = c.shared.s1;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: shared_resources::s2 = c.shared.s2;

        // `&` if read-only
        let _: &u32 = c.shared.s3;
    }

    #[task(binds = UART1, shared = [s2, &o5])]
    fn uart1(c: uart1::Context) {
        // owned by interrupt == `&` if read-only
        let _: &u32 = c.shared.o5;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: shared_resources::s2 = c.shared.s2;
    }
}
