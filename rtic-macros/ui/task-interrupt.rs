#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(binds = SysTick)]
    fn foo(_: foo::Context) {}

    #[task]
    async fn foo(_: foo::Context) {}
}
