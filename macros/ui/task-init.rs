#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn foo(_: foo::Context) -> (Shared, Local, foo::Monotonics) {}

    // name collides with `#[idle]` function
    #[task]
    fn foo(_: foo::Context) {}
}
