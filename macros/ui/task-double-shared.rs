#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(shared = [A], shared = [B])]
    fn foo(_: foo::Context) {}
}
