#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {}

    #[task(priority = 0)]
    fn foo(_: foo::Context) {}

    #[idle]
    fn idle(_: idle::Context) -> ! {}
}
