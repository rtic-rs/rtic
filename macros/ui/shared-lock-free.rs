#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {
        // An exclusive, early resource
        #[lock_free]
        e1: u32,

        // An exclusive, late resource
        #[lock_free]
        e2: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    // e2 ok
    #[idle(shared = [e2])]
    fn idle(cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);
        loop {}
    }

    // e1 rejected (not lock_free)
    #[task(priority = 1, shared = [e1])]
    fn uart0(cx: uart0::Context) {
        *cx.resources.e1 += 10;
    }

    // e1 rejected (not lock_free)
    #[task(priority = 2, shared = [e1])]
    fn uart1(cx: uart1::Context) {}
}
