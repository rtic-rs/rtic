//! examples/cancel-reschedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use systick_monotonic::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let systick = cx.core.SYST;

        // Initialize the monotonic (SysTick rate in QEMU is 12 MHz)
        let mono = Systick::new(systick, 12_000_000);

        hprintln!("init").ok();

        // Schedule `foo` to run 1 second in the future
        foo::spawn_after(1.secs()).unwrap();

        (
            Shared {},
            Local {},
            init::Monotonics(mono), // Give the monotonic to RTIC
        )
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo").ok();

        // Schedule `bar` to run 2 seconds in the future (1 second after foo runs)
        let spawn_handle = baz::spawn_after(2.secs()).unwrap();
        bar::spawn_after(1.secs(), spawn_handle, false).unwrap(); // Change to true
    }

    #[task]
    fn bar(_: bar::Context, baz_handle: baz::SpawnHandle, do_reschedule: bool) {
        hprintln!("bar").ok();

        if do_reschedule {
            // Reschedule baz 2 seconds from now, instead of the original 1 second
            // from now.
            baz_handle.reschedule_after(2.secs()).unwrap();
            // Or baz_handle.reschedule_at(/* time */)
        } else {
            // Or cancel it
            baz_handle.cancel().unwrap();
            debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        }
    }

    #[task]
    fn baz(_: baz::Context) {
        hprintln!("baz").ok();
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
