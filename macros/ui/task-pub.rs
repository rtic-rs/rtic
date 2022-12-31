#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task]
    pub fn foo(_: foo::Context) {}
}
