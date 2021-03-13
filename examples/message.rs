//! examples/message.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[resources]
    struct Resources {
        #[task_local]
        #[init(0)]
        times: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        foo::spawn(/* no message */).unwrap();

        (init::LateResources {}, init::Monotonics())
    }

    #[task(resources = [times])]
    fn foo(cx: foo::Context) {
        let times = cx.resources.times;

        hprintln!("foo").unwrap();

        bar::spawn(*times).unwrap();
        *times += 1;
    }

    #[task]
    fn bar(_: bar::Context, x: u32) {
        hprintln!("bar({})", x).unwrap();

        baz::spawn(x + 1, x + 2).unwrap();
    }

    #[task]
    fn baz(_: baz::Context, x: u32, y: u32) {
        hprintln!("baz({}, {})", x, y).unwrap();

        if x + y > 4 {
            debug::exit(debug::EXIT_SUCCESS);
        }

        foo::spawn().unwrap();
    }
}
