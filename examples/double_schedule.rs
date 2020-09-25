//! examples/double_schedule.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;
use rtic::cyccnt::U32Ext;

#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        nothing: (),
    }

    #[init(spawn = [task1])]
    fn init(cx: init::Context) -> init::LateResources {
        cx.spawn.task1().ok();

        init::LateResources { nothing: () }
    }

    #[task(schedule = [task2])]
    fn task1(_cx: task1::Context) {
        _cx.schedule.task2(_cx.scheduled + 100.cycles()).ok();
    }

    #[task(schedule = [task1])]
    fn task2(_cx: task2::Context) {
        _cx.schedule.task1(_cx.scheduled + 100.cycles()).ok();
    }

    extern "C" {
        fn SSI0();
    }
};
