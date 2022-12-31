#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(local = [A], local = [B])]
    fn foo(_: foo::Context) {}
}
