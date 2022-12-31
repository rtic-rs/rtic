#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {
        #[lock_free]
        e: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    // e ok
    #[task(priority = 1, shared = [e])]
    fn uart0(cx: uart0::Context) {}

    // e ok
    #[task(priority = 1, shared = [e])]
    fn uart1(cx: uart1::Context) {}

    // e not ok
    #[task(priority = 1, shared = [e])]
    async fn async_task(cx: async_task::Context) {}
}
