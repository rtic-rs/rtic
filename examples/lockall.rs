//! examples/lock.rs

// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        a: u32,
        b: i64,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared { a: 1, b: 2 }, Local {}, init::Monotonics())
    }

    // when omitted priority is assumed to be `1`
    #[task(shared = [a, b])]
    fn foo(mut c: foo::Context) {
        static mut X: Option<&'static mut u32> = None;
        static mut Y: u32 = 0;
        let _ = hprintln!("before lock");
        c.shared.lock(|s| {
            let _ = hprintln!("in lock");
            let _ = hprintln!("here {}, {}", s.a, s.b);
            *s.a += 1;

            // soundness check
            // c.shared.lock(|s| {}); // borrow error
            // c.shared.a.lock(|s| {}); // borrow error

            unsafe {
                X = Some(&mut Y);
                // X = Some(s.a); // lifetime issue
                // X = Some(&mut *s.a); // lifetime issue
                // X = Some(&'static mut *s.a); // not rust
            }
            let _ = hprintln!("here {}, {}", s.a, s.b);
        });
        // the lower priority task requires a critical section to access the data
        // c.shared.shared.lock(|shared| {
        //     // data can only be modified within this critical section (closure)
        //     *shared += 1;

        //     // bar will *not* run right now due to the critical section
        //     bar::spawn().unwrap();

        //     hprintln!("B - shared = {}", *shared).unwrap();

        //     // baz does not contend for `shared` so it's allowed to run now
        //     baz::spawn().unwrap();
        // });

        // // critical section is over: bar can now start

        // hprintln!("E").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
