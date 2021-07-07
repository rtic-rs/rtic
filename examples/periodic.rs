//! examples/periodic.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

// NOTE: does NOT work on QEMU!
#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use dwt_systick_monotonic::DwtSystick;
    use rtic::time::duration::Seconds;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<8_000_000>; // 8 MHz

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        let mono = DwtSystick::new(&mut dcb, dwt, systick, 8_000_000);

        foo::spawn_after(Seconds(1_u32)).unwrap();

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[task]
    fn foo(_cx: foo::Context) {
        // Periodic
        foo::spawn_after(Seconds(1_u32)).unwrap();
    }
}
