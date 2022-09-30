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

    #[derive(Default)]
    struct M {
        must_not_be_cloned: bool,
    }

    impl Clone for M {
        fn clone(&self) -> Self {
            assert!(!self.must_not_be_cloned);
            M {
                must_not_be_cloned: self.must_not_be_cloned,
            }
        }
    }

    impl Receive<M> for A {
        fn receive(&mut self, _: M) {
            static WAS_CALLED_EXACTLY_ONCE: AtomicBool = AtomicBool::new(false);
            hprintln!("A::receive was called").ok();
            assert!(!WAS_CALLED_EXACTLY_ONCE.load(Ordering::Relaxed));
            WAS_CALLED_EXACTLY_ONCE.store(true, Ordering::Relaxed);
        }
    }

    impl Receive<M> for B {
        fn receive(&mut self, _: M) {
            static WAS_CALLED_EXACTLY_ONCE: AtomicBool = AtomicBool::new(false);
            hprintln!("B::receive was called").ok();
            assert!(!WAS_CALLED_EXACTLY_ONCE.load(Ordering::Relaxed));
            WAS_CALLED_EXACTLY_ONCE.store(true, Ordering::Relaxed);
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
        assert!(poster.post(M::default()).is_ok());

        // B's message queue is full so message must NOT be cloned
        // this must also NOT trigger task A even if it has capacity
        assert!(poster
            .post(M {
                must_not_be_cloned: true
            })
            .is_err());

        (Shared {}, Local {}, init::Monotonics(), Actors {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            debug::exit(debug::EXIT_SUCCESS)
        }
    }

    #[local]
    struct Local {}

    #[shared]
    struct Shared {}
}
