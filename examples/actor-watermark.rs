// This example depends on the `memory-watermark` feature
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_actor_traits::Receive;

    struct Actor;

    struct Message;

    impl Receive<Message> for Actor {
        fn receive(&mut self, _: Message) {
            hprintln!("Actor::receive was called").ok();
        }
    }

    #[actors]
    struct Actors {
        #[subscribe(Message, capacity = 2)]
        actor: Actor,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
        assert_eq!(0, actor::SUBSCRIPTIONS[0].watermark());
        assert_eq!("Message", actor::SUBSCRIPTIONS[0].message_type);
        assert_eq!(2, actor::SUBSCRIPTIONS[0].capacity);

        assert!(cx.poster.post(Message).is_ok());
        assert_eq!(1, actor::SUBSCRIPTIONS[0].watermark());

        // monotonically increasing
        assert!(cx.poster.post(Message).is_ok());
        assert_eq!(2, actor::SUBSCRIPTIONS[0].watermark());

        // bounded value
        assert!(cx.poster.post(Message).is_err());
        assert_eq!(2, actor::SUBSCRIPTIONS[0].watermark());

        (
            Shared {},
            Local {},
            init::Monotonics(),
            Actors { actor: Actor },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // monotonically increasing: does not decrease after receive is called
        assert_eq!(2, actor::SUBSCRIPTIONS[0].watermark());

        loop {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}
}
