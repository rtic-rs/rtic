//! examples/schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

// NOTE: does NOT work on QEMU!
#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::hprintln;
    use dwt_systick_monotonic::DwtSystick;
    use rtic::time::duration::Seconds;

    const MONO_HZ: u32 = 8_000_000; // 8 MHz

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<MONO_HZ>;

    #[init()]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        let mono = DwtSystick::new(&mut dcb, dwt, systick, 8_000_000);

        hprintln!("init").ok();

        // Schedule `foo` to run 1 second in the future
        foo::spawn_after(Seconds(1_u32)).ok();

        // Schedule `bar` to run 2 seconds in the future
        bar::spawn_after(Seconds(2_u32)).ok();

        (init::LateResources {}, init::Monotonics(mono))
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo").ok();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!("bar").ok();
    }
}
