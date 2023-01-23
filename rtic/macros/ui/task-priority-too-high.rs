#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(priority = 256)]
    async fn foo(_: foo::Context) {}
}
