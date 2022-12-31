#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(capacity = 1, capacity = 2)]
    fn foo(_: foo::Context) {}
}
