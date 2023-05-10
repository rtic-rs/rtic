#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {}

    #[idle]
    fn idle(_: idle::Context) -> ! {}

    #[task]
    async fn task1(_: task1::Context) {}
}
