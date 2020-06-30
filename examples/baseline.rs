//! examples/baseline.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

// NOTE: does NOT properly work on QEMU
#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    #[init(spawn = [foo])]
    fn init(cx: init::Context) {
        // omitted: initialization of `CYCCNT`

        hprintln!("init(baseline = {:?})", cx.start).unwrap();

        // `foo` inherits the baseline of `init`: `Instant(0)`
        cx.spawn.foo().unwrap();
    }

    #[task(schedule = [foo])]
    fn foo(cx: foo::Context) {
        static mut ONCE: bool = true;

        hprintln!("foo(baseline = {:?})", cx.scheduled).unwrap();

        if *ONCE {
            *ONCE = false;

            rtic::pend(Interrupt::UART0);
        } else {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[task(binds = UART0, spawn = [foo])]
    fn uart0(cx: uart0::Context) {
        hprintln!("UART0(baseline = {:?})", cx.start).unwrap();

        // `foo` inherits the baseline of `UART0`: its `start` time
        cx.spawn.foo().unwrap();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
};
