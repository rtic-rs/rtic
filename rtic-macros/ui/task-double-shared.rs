#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(shared = [A], shared = [B])]
    async fn foo(_: foo::Context) {}
}
