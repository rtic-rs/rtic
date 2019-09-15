//! [compile-pass] Check code generation of resources

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
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

    #[init(resources = [o1, o4, o5, o6, s3])]
    fn init(c: init::Context) {
        // owned by `init` == `&'static mut`
        let _: &'static mut u32 = c.resources.o1;

        // owned by `init` == `&'static` if read-only
        let _: &'static u32 = c.resources.o6;

        // `init` has exclusive access to all resources
        let _: &mut u32 = c.resources.o4;
        let _: &mut u32 = c.resources.o5;
        let _: &mut u32 = c.resources.s3;
    }

    #[idle(resources = [o2, &o4, s1, &s3])]
    fn idle(mut c: idle::Context) -> ! {
        // owned by `idle` == `&'static mut`
        let _: &'static mut u32 = c.resources.o2;

        // owned by `idle` == `&'static` if read-only
        let _: &'static u32 = c.resources.o4;

        // shared with `idle` == `Mutex`
        c.resources.s1.lock(|_| {});

        // `&` if read-only
        let _: &u32 = c.resources.s3;

        loop {}
    }

    #[task(binds = UART0, resources = [o3, s1, s2, &s3])]
    fn uart0(c: uart0::Context) {
        // owned by interrupt == `&mut`
        let _: &mut u32 = c.resources.o3;

        // no `Mutex` proxy when access from highest priority task
        let _: &mut u32 = c.resources.s1;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: &mut u32 = c.resources.s2;

        // `&` if read-only
        let _: &u32 = c.resources.s3;
    }

    #[task(binds = UART1, resources = [s2, &o5])]
    fn uart1(c: uart1::Context) {
        // owned by interrupt == `&` if read-only
        let _: &u32 = c.resources.o5;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: &mut u32 = c.resources.s2;
    }
};
