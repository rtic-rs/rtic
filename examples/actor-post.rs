#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use core::sync::atomic::{AtomicBool, Ordering};

    use cortex_m_semihosting::{debug, hprintln};
    use rtic_actor_traits::Receive;

    struct Actor;

    const PAYLOAD: i32 = 42;

    struct Message {
        payload: i32,
    }

    static RECEIVE_WAS_CALLED: AtomicBool = AtomicBool::new(false);

    impl Receive<Message> for Actor {
        fn receive(&mut self, m: Message) {
            hprintln!("Actor::receive was called").ok();

            RECEIVE_WAS_CALLED.store(true, Ordering::Relaxed);

            assert_eq!(PAYLOAD, m.payload);
        }
    }

    #[actors]
    struct Actors {
        #[subscribe(Message)]
        actor: Actor,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
        cx.poster.post(Message { payload: PAYLOAD }).ok();

        // receive invocation withheld
        assert!(!RECEIVE_WAS_CALLED.load(Ordering::Relaxed));

        (
            Shared {},
            Local {},
            init::Monotonics(),
            Actors { actor: Actor },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // receive invocation must have executed by now
        assert!(RECEIVE_WAS_CALLED.load(Ordering::Relaxed));

        loop {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}
}
