#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[init(shared = [A], shared = [B])]
    fn init(_: init::Context) {}
}
