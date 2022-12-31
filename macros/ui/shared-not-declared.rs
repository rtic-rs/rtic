#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[task(shared = [A])]
    fn foo(_: foo::Context) {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}
}
