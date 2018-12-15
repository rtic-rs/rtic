#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate owned_singleton;
extern crate panic_halt;
extern crate rtfm;

use rtfm::{app, Exclusive};

#[app(device = lm3s6965)]
const APP: () = {
    #[Singleton]
    static mut O1: u32 = 0;
    #[Singleton]
    static mut O2: u32 = 0;
    #[Singleton]
    static mut O3: u32 = 0;
    #[Singleton]
    static O4: u32 = 0;
    #[Singleton]
    static O5: u32 = 0;
    #[Singleton]
    static O6: u32 = 0;

    #[Singleton]
    static mut S1: u32 = 0;
    #[Singleton]
    static S2: u32 = 0;

    #[init(resources = [O1, O2, O3, O4, O5, O6, S1, S2])]
    fn init() {
        let _: O1 = resources.O1;
        let _: &mut O2 = resources.O2;
        let _: &mut O3 = resources.O3;
        let _: O4 = resources.O4;
        let _: &mut O5 = resources.O5;
        let _: &mut O6 = resources.O6;

        let _: &mut S1 = resources.S1;
        let _: &mut S2 = resources.S2;
    }

    #[idle(resources = [O2, O5])]
    fn idle() -> ! {
        let _: O2 = resources.O2;
        let _: O5 = resources.O5;

        loop {}
    }

    #[interrupt(resources = [O3, O6, S1, S2])]
    fn UART0() {
        let _: &mut O3 = resources.O3;
        let _: &O6 = resources.O6;

        let _: Exclusive<S1> = resources.S1;
        let _: &S2 = resources.S2;
    }

    #[interrupt(resources = [S1, S2])]
    fn UART1() {
        let _: Exclusive<S1> = resources.S1;
        let _: &S2 = resources.S2;
    }
};
