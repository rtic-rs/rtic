//! examples/static-resources-in-divergent.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;
use rtic_monotonics::systick::prelude::*;
systick_monotonic!(Mono, 100);

#[rtic::app(device = lm3s6965, dispatchers = [UART0])]
mod app {
    use super::*;

    use cortex_m_semihosting::{debug, hprintln};
    use rtic_sync::channel::{Channel, Receiver};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        Mono::start(cx.core.SYST, 12_000_000);

        divergent::spawn().ok();

        (Shared {}, Local {})
    }

    #[task(local = [q: Channel<u32, 5> = Channel::new()], priority = 1)]
    async fn divergent(cx: divergent::Context) -> ! {
        // `q` has `'static` lifetime. You can put references to it in `static` variables,
        // structs with references to `q` do not need a generic lifetime parameter, etc.

        let (mut tx, rx) = cx.local.q.split();
        let mut state = 0;

        bar::spawn(rx).unwrap();

        loop {
            tx.send(state).await.unwrap();
            state += 1;
            Mono::delay(100.millis()).await;
        }
    }

    #[task(priority = 1)]
    async fn bar(_cx: bar::Context, mut rx: Receiver<'static, u32, 5>) -> ! {
        loop {
            // Lock-free access to the same underlying queue!
            if let Some(data) = rx.recv().await.ok() {
                hprintln!("received message: {}", data);

                if data == 3 {
                    debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
                } else {
                    Mono::delay(100.millis()).await;
                }
            }
        }
    }
}
