#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {}

    #[task(local = [
        #[test]
        a: u32 = 0, // Ok
        #[test]
        b, // Error
    ])]
    fn foo(_: foo::Context) {}
}
