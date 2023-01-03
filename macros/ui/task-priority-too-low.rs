#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(binds = UART0, priority = 0)]
    fn foo(_: foo::Context) {}
}
