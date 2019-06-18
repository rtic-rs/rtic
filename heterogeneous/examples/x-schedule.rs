#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(cores = 2, device = heterogeneous, monotonic = heterogeneous::MT)]
const APP: () = {
    #[init(core = 0, spawn = [ping])]
    fn init(c: init::Context) {
        c.spawn.ping().ok();
    }

    #[task(core = 0, schedule = [ping])]
    fn pong(c: pong::Context) {
        c.schedule.ping(c.scheduled + 1_000_000).ok();
    }

    #[task(core = 1, schedule = [pong])]
    fn ping(c: ping::Context) {
        c.schedule.pong(c.scheduled + 1_000_000).ok();
    }

    extern "C" {
        #[core = 0]
        fn I0();

        #[core = 0]
        fn I1();

        #[core = 1]
        fn I0();

        #[core = 1]
        fn I1();
    }
};
