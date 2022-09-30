#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use core::sync::atomic::{AtomicU8, Ordering};

    use cortex_m_semihosting::{debug, hprintln};
    use rtic_actor_traits::Receive;

    struct Actor;

    struct Message;

    static CALL_COUNT: AtomicU8 = AtomicU8::new(0);

    impl Receive<Message> for Actor {
        fn receive(&mut self, _: Message) {
            hprintln!("Actor::receive was called").ok();
            CALL_COUNT.store(CALL_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
        }
    }

    #[actors]
    struct Actors {
        #[subscribe(Message, capacity = 2)]
        actor: Actor,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
        assert!(cx.poster.post(Message).is_ok());
        assert!(cx.poster.post(Message).is_ok());
        assert!(cx.poster.post(Message).is_err());

        (
            Shared {},
            Local {},
            init::Monotonics(),
            Actors { actor: Actor },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        assert_eq!(2, CALL_COUNT.load(Ordering::Relaxed));

        loop {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}
}
