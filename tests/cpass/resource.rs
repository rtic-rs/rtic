//! Check code generation of resources

#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::{app, Exclusive};

#[app(device = lm3s6965)]
const APP: () = {
    static mut O1: u32 = 0; // init
    static mut O2: u32 = 0; // idle
    static mut O3: u32 = 0; // EXTI0
    static O4: u32 = 0; // idle
    static O5: u32 = 0; // EXTI1
    static O6: u32 = 0; // init

    static mut S1: u32 = 0; // idle & EXTI0
    static mut S2: u32 = 0; // EXTI0 & EXTI1
    static S3: u32 = 0;

    #[init(resources = [O1, O4, O5, O6, S3])]
    fn init() {
        // owned by `init` == `&'static mut`
        let _: &'static mut u32 = resources.O1;

        // owned by `init` == `&'static` if read-only
        let _: &'static u32 = resources.O6;

        // `init` has exclusive access to all resources
        let _: &mut u32 = resources.O4;
        let _: &mut u32 = resources.O5;
        let _: &mut u32 = resources.S3;
    }

    #[idle(resources = [O2, O4, S1, S3])]
    fn idle() -> ! {
        // owned by `idle` == `&'static mut`
        let _: &'static mut u32 = resources.O2;

        // owned by `idle` == `&'static` if read-only
        let _: &'static u32 = resources.O4;

        // shared with `idle` == `Mutex`
        resources.S1.lock(|_| {});

        // `&` if read-only
        let _: &u32 = resources.S3;

        loop {}
    }

    #[interrupt(resources = [O3, S1, S2, S3])]
    fn UART0() {
        // owned by interrupt == `&mut`
        let _: &mut u32 = resources.O3;

        // no `Mutex` proxy when access from highest priority task
        let _: Exclusive<u32> = resources.S1;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: Exclusive<u32> = resources.S2;

        // `&` if read-only
        let _: &u32 = resources.S3;
    }

    #[interrupt(resources = [S2, O5])]
    fn UART1() {
        // owned by interrupt == `&` if read-only
        let _: &u32 = resources.O5;

        // no `Mutex` proxy when co-owned by cooperative (same priority) tasks
        let _: Exclusive<u32> = resources.S2;
    }
};
