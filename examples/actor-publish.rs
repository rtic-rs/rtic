#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA, GPIOB])]
mod app {
    use core::sync::atomic::{AtomicBool, Ordering};

    use cortex_m_semihosting::{debug, hprintln};
    use rtic_actor_traits::Receive;

    struct A;
    struct B;

    const PAYLOAD: i32 = 42;

    static CLONE_WAS_CALLED_EXACTLY_ONCE: AtomicBool = AtomicBool::new(false);

    struct M {
        payload: i32,
    }

    impl Clone for M {
        fn clone(&self) -> Self {
            assert!(!CLONE_WAS_CALLED_EXACTLY_ONCE.load(Ordering::Relaxed));
            CLONE_WAS_CALLED_EXACTLY_ONCE.store(true, Ordering::Relaxed);

            // `derive(Clone)` implementation
            Self {
                payload: self.payload.clone(),
            }
        }
    }

    static A_RECEIVE_WAS_CALLED: AtomicBool = AtomicBool::new(false);

    impl Receive<M> for A {
        fn receive(&mut self, m: M) {
            hprintln!("A::receive was called").ok();

            assert_eq!(PAYLOAD, m.payload);
            A_RECEIVE_WAS_CALLED.store(true, Ordering::Relaxed);
        }
    }

    static B_RECEIVE_WAS_CALLED: AtomicBool = AtomicBool::new(false);

    impl Receive<M> for B {
        fn receive(&mut self, m: M) {
            hprintln!("B::receive was called").ok();

            assert_eq!(PAYLOAD, m.payload);
            B_RECEIVE_WAS_CALLED.store(true, Ordering::Relaxed);
        }
    }

    #[actors]
    struct Actors {
        #[subscribe(M, capacity = 2)]
        #[init(A)]
        a: A,

        #[subscribe(M, capacity = 1)]
        #[init(B)]
        b: B,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
        let mut poster = cx.poster;
        assert!(poster.post(M { payload: PAYLOAD }).is_ok());

        assert!(CLONE_WAS_CALLED_EXACTLY_ONCE.load(Ordering::Relaxed));

        // receive invocations withheld
        assert!(!A_RECEIVE_WAS_CALLED.load(Ordering::Relaxed));
        assert!(!B_RECEIVE_WAS_CALLED.load(Ordering::Relaxed));

        (Shared {}, Local {}, init::Monotonics(), Actors {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // receive invocations must have executed by now
        assert!(A_RECEIVE_WAS_CALLED.load(Ordering::Relaxed));
        assert!(B_RECEIVE_WAS_CALLED.load(Ordering::Relaxed));

        loop {
            debug::exit(debug::EXIT_SUCCESS)
        }
    }

    #[local]
    struct Local {}

    #[shared]
    struct Shared {}
}
