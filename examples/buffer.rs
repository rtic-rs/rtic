#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [foo])]
    fn init() {
        spawn.foo(0).unwrap();
    }

    #[idle]
    fn idle() -> ! {
        hprintln!("idle").unwrap();

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }

    #[task]
    fn foo(m: Message<u32>) {
        // HACK: `mut m: Message<u32>` in the function signature doesn't work properly
        let mut m = m;

        hprintln!("{}", *m).unwrap();

        *m += 1;

        hprintln!("{}", *m).unwrap();

        // NOTE: consumes `m`
        let x: u32 = m.read();

        hprintln!("{}", x).unwrap();

        // hprintln!("{}", *m).unwrap(); //~ ERROR: `m` has been moved
    }

    // You can still write tasks like this
    #[task]
    fn bar(_x: u32) {}

    // or this
    #[task]
    fn baz(_x: u32, _y: u32) {}

    extern "C" {
        fn UART0();
    }
};
