//! examples/schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

// NOTE: does NOT work on QEMU!
#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::hprintln;
    use dwt_systick_monotonic::{
        consts::{U0, U8},
        DwtSystick,
    };
    use rtic::time::duration::Seconds;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<U8, U0, U0>; // 8 MHz

    #[init()]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        let mono = DwtSystick::new(&mut dcb, dwt, systick, 8_000_000);

        hprintln!("init").unwrap();

        // Schedule `foo` to run 1 second in the future
        foo::spawn_after(Seconds(1_u32)).unwrap();

        // Schedule `bar` to run 2 seconds in the future
        bar::spawn_after(Seconds(2_u32)).unwrap();

        (init::LateResources {}, init::Monotonics(mono))
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo").unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!("bar").unwrap();
    }
}
