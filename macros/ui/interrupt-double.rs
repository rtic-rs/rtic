#![no_main]

#[rtic_macros::mock_app(parse_binds, device = mock)]
mod app {
    #[task(binds = UART0)]
    fn foo(_: foo::Context) {}

    #[task(binds = UART0)]
    fn bar(_: bar::Context) {}
}
