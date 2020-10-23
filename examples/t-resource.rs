//! [compile-pass] Check code generation of resources

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[resources]
    struct Resources {
        #[init(0)]
        o1: u32, // init
        #[init(0)]
        o2: u32, // idle
        #[init(0)]
        o3: u32, // EXTI0
        #[init(0)]
        o4: u32, // idle
        #[init(0)]
        o5: u32, // EXTI1
        #[init(0)]
        o6: u32, // init
        #[init(0)]
        s1: u32, // idle & uart0
        #[init(0)]
        s2: u32, // uart0 & uart1
        #[init(0)]
        s3: u32, // idle & uart0
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        init::LateResources {}
    }

    #[idle(resources = [o2, &o4, s1, &s3])]
    fn idle(mut c: idle::Context) -> ! {
        // owned by `idle` == `&'static mut`
        let _: resources::o2 = c.resources.o2;

        // owned by `idle` == `&'static` if read-only
        let _: &u32 = c.resources.o4;

        // shared with `idle` == `Mutex`
        c.resources.s1.lock(|_| {});

        // `&` if read-only
        let _: &u32 = c.resources.s3;

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = UART0, resources = [o3, s1, s2, &s3])]
    fn uart0(c: uart0::Context) {
        // owned by interrupt == `&mut`
        let _: resources::o3 = c.resources.o3;

        // no `Mutex` proxy when access from highest priority task
        let _: resources::s1 = c.resources.s1;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: resources::s2 = c.resources.s2;

        // `&` if read-only
        let _: &u32 = c.resources.s3;
    }

    #[task(binds = UART1, resources = [s2, &o5])]
    fn uart1(c: uart1::Context) {
        // owned by interrupt == `&` if read-only
        let _: &u32 = c.resources.o5;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: resources::s2 = c.resources.s2;
    }
}
