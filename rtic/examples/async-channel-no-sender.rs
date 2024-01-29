//! examples/async-channel-no-sender.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_sync::{channel::*, make_channel};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    const CAPACITY: usize = 1;
    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let (_s, r) = make_channel!(u32, CAPACITY);

        receiver::spawn(r).unwrap();

        (Shared {}, Local {})
    }

    #[task]
    async fn receiver(_c: receiver::Context, mut receiver: Receiver<'static, u32, CAPACITY>) {
        hprintln!("Receiver got: {:?}", receiver.recv().await);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
