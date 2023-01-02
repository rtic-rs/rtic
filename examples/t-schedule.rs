//! [compile-pass] Check `schedule` code generation

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::debug;
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

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // Task without message passing

        // Not default
        let _: Result<foo::MyMono::SpawnHandle, ()> =
            foo::MyMono::spawn_at(monotonics::MyMono::now());
        let handle: Result<foo::MyMono::SpawnHandle, ()> = foo::MyMono::spawn_after(1.secs());
        let _: Result<foo::MyMono::SpawnHandle, ()> = handle.unwrap().reschedule_after(1.secs());

        let handle: Result<foo::MyMono::SpawnHandle, ()> = foo::MyMono::spawn_after(1.secs());
        let _: Result<foo::MyMono::SpawnHandle, ()> =
            handle.unwrap().reschedule_at(monotonics::MyMono::now());

        let handle: Result<foo::MyMono::SpawnHandle, ()> = foo::MyMono::spawn_after(1.secs());
        let _: Result<(), ()> = handle.unwrap().cancel();

        // Using default
        let _: Result<foo::SpawnHandle, ()> = foo::spawn_at(monotonics::now());
        let handle: Result<foo::SpawnHandle, ()> = foo::spawn_after(1.secs());
        let _: Result<foo::SpawnHandle, ()> = handle.unwrap().reschedule_after(1.secs());

        let handle: Result<foo::SpawnHandle, ()> = foo::spawn_after(1.secs());
        let _: Result<foo::SpawnHandle, ()> =
            handle.unwrap().reschedule_at(monotonics::MyMono::now());

        let handle: Result<foo::SpawnHandle, ()> = foo::spawn_after(1.secs());
        let _: Result<(), ()> = handle.unwrap().cancel();

        // Task with single message passing

        // Not default
        let _: Result<bar::MyMono::SpawnHandle, u32> =
            bar::MyMono::spawn_at(monotonics::MyMono::now(), 0);
        let handle: Result<bar::MyMono::SpawnHandle, u32> = bar::MyMono::spawn_after(1.secs(), 1);
        let _: Result<bar::MyMono::SpawnHandle, ()> = handle.unwrap().reschedule_after(1.secs());

        let handle: Result<bar::MyMono::SpawnHandle, u32> = bar::MyMono::spawn_after(1.secs(), 1);
        let _: Result<bar::MyMono::SpawnHandle, ()> =
            handle.unwrap().reschedule_at(monotonics::MyMono::now());

        let handle: Result<bar::MyMono::SpawnHandle, u32> = bar::MyMono::spawn_after(1.secs(), 1);
        let _: Result<u32, ()> = handle.unwrap().cancel();

        // Using default
        let _: Result<bar::SpawnHandle, u32> = bar::spawn_at(monotonics::MyMono::now(), 0);
        let handle: Result<bar::SpawnHandle, u32> = bar::spawn_after(1.secs(), 1);
        let _: Result<bar::SpawnHandle, ()> = handle.unwrap().reschedule_after(1.secs());

        let handle: Result<bar::SpawnHandle, u32> = bar::spawn_after(1.secs(), 1);
        let _: Result<bar::SpawnHandle, ()> =
            handle.unwrap().reschedule_at(monotonics::MyMono::now());

        let handle: Result<bar::SpawnHandle, u32> = bar::spawn_after(1.secs(), 1);
        let _: Result<u32, ()> = handle.unwrap().cancel();

        // Task with multiple message passing

        // Not default
        let _: Result<baz::MyMono::SpawnHandle, (u32, u32)> =
            baz::MyMono::spawn_at(monotonics::MyMono::now(), 0, 1);
        let handle: Result<baz::MyMono::SpawnHandle, (u32, u32)> =
            baz::MyMono::spawn_after(1.secs(), 1, 2);
        let _: Result<baz::MyMono::SpawnHandle, ()> = handle.unwrap().reschedule_after(1.secs());

        let handle: Result<baz::MyMono::SpawnHandle, (u32, u32)> =
            baz::MyMono::spawn_after(1.secs(), 1, 2);
        let _: Result<baz::MyMono::SpawnHandle, ()> =
            handle.unwrap().reschedule_at(monotonics::MyMono::now());

        let handle: Result<baz::MyMono::SpawnHandle, (u32, u32)> =
            baz::MyMono::spawn_after(1.secs(), 1, 2);
        let _: Result<(u32, u32), ()> = handle.unwrap().cancel();

        // Using default
        let _: Result<baz::SpawnHandle, (u32, u32)> =
            baz::spawn_at(monotonics::MyMono::now(), 0, 1);
        let handle: Result<baz::SpawnHandle, (u32, u32)> = baz::spawn_after(1.secs(), 1, 2);
        let _: Result<baz::SpawnHandle, ()> = handle.unwrap().reschedule_after(1.secs());

        let handle: Result<baz::SpawnHandle, (u32, u32)> = baz::spawn_after(1.secs(), 1, 2);
        let _: Result<baz::SpawnHandle, ()> =
            handle.unwrap().reschedule_at(monotonics::MyMono::now());

        let handle: Result<baz::SpawnHandle, (u32, u32)> = baz::spawn_after(1.secs(), 1, 2);
        let _: Result<(u32, u32), ()> = handle.unwrap().cancel();

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
