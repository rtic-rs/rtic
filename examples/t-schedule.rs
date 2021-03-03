//! [compile-pass] Check `schedule` code generation

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
        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        let mono = DwtSystick::new(&mut dcb, dwt, systick, 8_000_000);

        let _: Result<(), ()> = foo::spawn_after(Seconds(1_u32));
        let _: Result<(), u32> = bar::spawn_after(Seconds(2_u32), 0);
        let _: Result<(), (u32, u32)> = baz::spawn_after(Seconds(3_u32), 0, 1);

        (init::LateResources {}, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let _: Result<(), ()> = foo::spawn_at(MyMono::now() + Seconds(3_u32));
        let _: Result<(), u32> = bar::spawn_at(MyMono::now() + Seconds(4_u32), 0);
        let _: Result<(), (u32, u32)> = baz::spawn_at(MyMono::now() + Seconds(5_u32), 0, 1);

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task]
    fn foo(_: foo::Context) {}

    #[task]
    fn bar(_: bar::Context, _x: u32) {}

    #[task]
    fn baz(_: baz::Context, _x: u32, _y: u32) {}
}
