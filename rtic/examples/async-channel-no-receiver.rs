//! examples/async-channel-no-receiver.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_channel::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    const CAPACITY: usize = 1;
    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let (s, _r) = make_channel!(u32, CAPACITY);

        sender1::spawn(s.clone()).unwrap();

        (Shared {}, Local {})
    }

    
    #[task]
    async fn sender1(_c: sender1::Context, mut sender: Sender<'static, u32, CAPACITY>) {

        hprintln!("Sender 1 sending: 1 {:?}", sender.send(1).await);
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

}