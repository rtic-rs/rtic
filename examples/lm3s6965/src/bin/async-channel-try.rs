//! examples/async-channel-try.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_sync::{channel::*, make_channel};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        sender: Sender<'static, u32, CAPACITY>,
    }

    const CAPACITY: usize = 1;
    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let (s, r) = make_channel!(u32, CAPACITY);

        receiver::spawn(r).unwrap();
        sender1::spawn(s.clone()).unwrap();

        (Shared {}, Local { sender: s.clone() })
    }

    #[task]
    async fn receiver(_c: receiver::Context, mut receiver: Receiver<'static, u32, CAPACITY>) {
        while let Ok(val) = receiver.recv().await {
            hprintln!("Receiver got: {}", val);
        }
    }

    #[task]
    async fn sender1(_c: sender1::Context, mut sender: Sender<'static, u32, CAPACITY>) {
        hprintln!("Sender 1 sending: 1");
        sender.send(1).await.unwrap();
        hprintln!("Sender 1 try sending: 2 {:?}", sender.try_send(2));
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    // This interrupt is never triggered, but is used to demonstrate that
    // one can (try to) send data into a channel from a hardware task.
    #[task(binds = GPIOA, local = [sender])]
    fn hw_task(cx: hw_task::Context) {
        cx.local.sender.try_send(3).ok();
    }
}
