#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn foo(_: foo::Context) -> (Shared, Local) {}

    // name collides with `#[idle]` function
    #[task]
    async fn foo(_: foo::Context) {}
}
