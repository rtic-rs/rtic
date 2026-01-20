//! examples/async-watch.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_sync::{watch::*, make_watch};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let (s, r) = make_watch!(u32);

        receiver::spawn(r).unwrap();
        sender::spawn(s.clone()).unwrap();

        (Shared {}, Local {})
    }

    #[task]
    async fn receiver(_c: receiver::Context, mut receiver: WatchReader<'static, u32>) {
        let val = receiver.changed().await;
        hprintln!("Receiver got: {}", val);
        let val = receiver.try_get();
        hprintln!("Receiver got: {:?}", val);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task]
    async fn sender(_c: sender::Context, mut sender: WatchWriter<'static, u32>) {
        hprintln!("Sender 1 writing: 1");
        sender.write(1);
    }
}
