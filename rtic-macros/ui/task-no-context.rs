#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task]
    async fn foo() {}
}
