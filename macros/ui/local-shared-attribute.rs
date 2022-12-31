#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(local = [
        #[test]
        a: u32 = 0, // Ok
        #[test]
        b, // Error
    ])]
    fn foo(_: foo::Context) {

    }
}
