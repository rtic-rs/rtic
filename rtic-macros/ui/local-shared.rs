#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        l1: u32,
        l2: u32,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {}

    // l2 ok
    #[idle(local = [l2])]
    fn idle(cx: idle::Context) -> ! {}

    // l1 rejected (not local)
    #[task(priority = 1, local = [l1])]
    async fn uart0(cx: uart0::Context) {}

    // l1 rejected (not lock_free)
    #[task(priority = 2, local = [l1])]
    async fn uart1(cx: uart1::Context) {}
}
