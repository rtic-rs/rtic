#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [UnknownInterrupt])]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        (Shared {}, Local {}, init::Monotonics())
    }
}
