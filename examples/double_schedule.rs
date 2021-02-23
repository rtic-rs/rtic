//! examples/double_schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use dwt_systick_monotonic::{
        consts::{U0, U8},
        DwtSystick,
    };
    use rtic::time::duration::Seconds;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<U8, U0, U0>; // 8 MHz

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        task1::spawn().ok();

        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        let mono = DwtSystick::new(&mut dcb, dwt, systick, 8_000_000);

        (init::LateResources {}, init::Monotonics(mono))
    }

    #[task]
    fn task1(_cx: task1::Context) {
        task2::spawn_after(Seconds(1_u32)).ok();
    }

    #[task]
    fn task2(_cx: task2::Context) {
        task1::spawn_after(Seconds(1_u32)).ok();
    }
}
