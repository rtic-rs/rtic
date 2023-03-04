#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task]
    pub async fn foo(_: foo::Context) {}
}
