#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[init(local = [A], local = [B])]
    fn init(_: init::Context) {}
}
