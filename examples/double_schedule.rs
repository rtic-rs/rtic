//! examples/double_schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT)]
mod app {
    use rtic::cyccnt::U32Ext;

    #[resources]
    struct Resources {
        nothing: (),
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        task1::spawn().ok();

        init::LateResources { nothing: () }
    }

    #[task]
    fn task1(_cx: task1::Context) {
        task2::schedule(_cx.scheduled + 100.cycles()).ok();
    }

    #[task]
    fn task2(_cx: task2::Context) {
        task1::schedule(_cx.scheduled + 100.cycles()).ok();
    }

    extern "C" {
        fn SSI0();
    }
}
