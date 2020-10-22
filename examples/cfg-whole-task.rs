//! examples/cfg-whole-task.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::debug;
    #[cfg(debug_assertions)]
    use cortex_m_semihosting::hprintln;

    #[resources]
    struct Resources {
        #[cfg(debug_assertions)] // <- `true` when using the `dev` profile
        #[init(0)]
        count: u32,
        #[cfg(never)]
        #[init(0)]
        unused: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        foo::spawn().unwrap();
        foo::spawn().unwrap();

        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(capacity = 2, resources = [count])]
    fn foo(_cx: foo::Context) {
        #[cfg(debug_assertions)]
        {
            *_cx.resources.count += 1;

            log::spawn(*_cx.resources.count).unwrap();
        }

        // this wouldn't compile in `release` mode
        // *_cx.resources.count += 1;

        // ..
    }

    // The whole task should disappear,
    // currently still present in the Tasks enum
    #[cfg(never)]
    #[task(capacity = 2, resources = [count])]
    fn foo2(_cx: foo2::Context) {
        #[cfg(debug_assertions)]
        {
            *_cx.resources.count += 10;

            log::spawn(*_cx.resources.count).unwrap();
        }

        // this wouldn't compile in `release` mode
        // *_cx.resources.count += 1;

        // ..
    }

    #[cfg(debug_assertions)]
    #[task(capacity = 2)]
    fn log(_: log::Context, n: u32) {
        hprintln!(
            "foo has been called {} time{}",
            n,
            if n == 1 { "" } else { "s" }
        )
        .ok();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
        fn QEI0();
    }
}
