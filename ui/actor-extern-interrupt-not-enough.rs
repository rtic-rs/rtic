#![no_main]

#[rtic::app(device = lm3s6965)]
mod app {
    #[actors]
    struct Actors {
        a: A,
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
        (Shared {}, Local {}, init::Monotonics {}, Actors { a: A })
    }
}
