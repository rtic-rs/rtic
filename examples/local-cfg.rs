//! examples/local.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

#[cfg(
    any(
        feature = "feature_s",
        feature = "feature_e1",
        feature = "feature_e2",
        feature = "feature_l1",
        feature = "feature_l2"
        )
    )
]
use cortex_m_semihosting::hprintln;
use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        // An early resource
        #[cfg(feature = "feature_s")]
        #[init(0)]
        shared: u32,

        // A local (move), early resource
        #[cfg(feature = "feature_l1")]
        #[task_local]
        #[init(1)]
        l1: u32,

        // An exclusive, early resource
        #[cfg(feature = "feature_e1")]
        #[lock_free]
        #[init(1)]
        e1: u32,

        // A local (move), late resource
        #[task_local]
        #[cfg(feature = "feature_l2")]
        l2: u32,

        // An exclusive, late resource
        #[cfg(feature = "feature_e2")]
        #[lock_free]
        e2: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);
        init::LateResources {
            #[cfg(feature = "feature_e2")]
            e2: 2,
            #[cfg(feature = "feature_l2")]
            l2: 2
        }
    }

    // `shared` cannot be accessed from this context
    // l1 ok (task_local)
    // e2 ok (lock_free)
    #[idle(resources =[#[cfg(feature = "feature_l1")]l1, e2])]
    fn idle(_cx: idle::Context) -> ! {
        #[cfg(feature = "feature_l1")]
        hprintln!("IDLE:l1 = {}", _cx.resources.l1).unwrap();
        #[cfg(feature = "feature_e2")]
        hprintln!("IDLE:e2 = {}", _cx.resources.e2).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {}
    }

    // `shared` can be accessed from this context
    // l2 ok (task_local)
    // e1 ok (lock_free)
    #[task(priority = 1, binds = UART0, resources = [shared, #[cfg(feature = "feature_l2")]l2, #[cfg(feature = "feature_e1")]e1])]
    fn uart0(_cx: uart0::Context) {
        #[cfg(feature = "feature_s")]
        let shared: &mut u32 = _cx.resources.shared;

        #[cfg(feature = "feature_s")]
        {
            *shared += 1;
        }

        #[cfg(feature = "feature_e1")]
        {
            // Unless put in a closure, the following error:
            // error[E0658]: attributes on expressions are experimental
            //   --> examples/local-cfg.rs:69:9
            //      69 |         #[cfg(feature = "feature_x")]
            //         |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
            //               = note: see issue #15701
            //               <https://github.com/rust-lang/rust/issues/15701> for more information
            *_cx.resources.e1 += 10;
        }
        #[cfg(feature = "feature_s")]
        hprintln!("UART0: shared = {}", shared).unwrap();
        #[cfg(feature = "feature_l2")]
        hprintln!("UART0:l2 = {}", _cx.resources.l2).unwrap();
        #[cfg(feature = "feature_e1")]
        hprintln!("UART0:e1 = {}", _cx.resources.e1).unwrap();
    }

    // `shared` can be accessed from this context
    // e1 ok (lock_free)
    #[task(priority = 1, binds = UART1, resources = [#[cfg(feature = "feature_s")]shared, #[cfg(feature = "feature_e1")]e1])]
    fn uart1(_cx: uart1::Context) {
        #[cfg(feature = "feature_s")]
        let shared: &mut u32 = _cx.resources.shared;
        #[cfg(feature = "feature_s")]
        {
            *shared += 1;
        }

        #[cfg(feature = "feature_s")]
        hprintln!("UART1: shared = {}", shared).unwrap();
        #[cfg(feature = "feature_e1")]
        hprintln!("UART1:e1 = {}", _cx.resources.e1).unwrap();
    }
};
