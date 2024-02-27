//! examples/extern_spawn.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

// Free function implementing the spawnable task `foo`.
// Notice, you need to indicate an anonymous lifetime <'a_>
async fn foo(_c: app::foo::Context<'_>) {
    hprintln!("foo");
    debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
}

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use crate::foo;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();

        (Shared {}, Local {})
    }

    extern "Rust" {
        #[task()]
        async fn foo(_c: foo::Context);
    }
}
