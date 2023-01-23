#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task]
    async unsafe fn foo(_: foo::Context) {}
}
