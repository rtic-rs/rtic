#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(local = [A], local = [B])]
    async fn foo(_: foo::Context) {}
}
