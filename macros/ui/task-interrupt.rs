#![no_main]

#[rtic_macros::mock_app(parse_binds, device = mock)]
mod app {
    #[task(binds = SysTick)]
    fn foo(_: foo::Context) {}

    #[task]
    fn foo(_: foo::Context) {}
}
