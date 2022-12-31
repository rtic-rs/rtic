#![no_main]

#[rtic_macros::mock_app(parse_binds, device = mock)]
mod app {
    #[task(binds = UART0, priority = 0)]
    fn foo(_: foo::Context) {}
}
