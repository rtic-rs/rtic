//! examples/task.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [foo])]
    fn init(c: init::Context) -> init::LateResources {
        c.spawn.foo().unwrap();

        init::LateResources {}
    }

    #[task(spawn = [bar, baz])]
    fn foo(c: foo::Context) {
        hprintln!("foo - start").unwrap();

        // spawns `bar` onto the task scheduler
        // `foo` and `bar` have the same priority so `bar` will not run until
        // after `foo` terminates
        c.spawn.bar().unwrap();

        hprintln!("foo - middle").unwrap();

        // spawns `baz` onto the task scheduler
        // `baz` has higher priority than `foo` so it immediately preempts `foo`
        c.spawn.baz().unwrap();

        hprintln!("foo - end").unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!("bar").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(priority = 2)]
    fn baz(_: baz::Context) {
        hprintln!("baz").unwrap();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
        fn QEI0();
    }
};
