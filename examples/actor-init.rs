#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_actor_traits::Receive;

    #[derive(Debug)]
    struct Actor {
        state: i32,
    }

    struct AssertActorWasInitialized;

    const INITIAL_STATE: i32 = 42;

    impl Receive<AssertActorWasInitialized> for Actor {
        fn receive(&mut self, _: AssertActorWasInitialized) {
            assert_eq!(INITIAL_STATE, self.state);
            hprintln!("OK").ok();
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[actors]
    struct Actors {
        #[subscribe(AssertActorWasInitialized)]
        #[init(Actor { state: INITIAL_STATE })]
        actor: Actor,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
        cx.poster.post(AssertActorWasInitialized).ok();

        (Shared {}, Local {}, init::Monotonics(), Actors {})
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}
}
