#![no_main]

#[rtic_macros::mock_app(parse_extern_interrupt, parse_binds, device = mock, dispatchers = [EXTI0])]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    #[task(binds = EXTI0)]
    fn foo(_: foo::Context) {}
}
