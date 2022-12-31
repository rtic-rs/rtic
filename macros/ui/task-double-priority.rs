#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(priority = 1, priority = 2)]
    fn foo(_: foo::Context) {}
}
