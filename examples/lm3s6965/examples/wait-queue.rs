//! examples/wait-queue.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_common::wait_queue::WaitQueue;

    use rtic_monotonics::systick::prelude::*;
    systick_monotonic!(Mono, 100);

    #[shared]
    struct Shared {
        count: u32,
    }

    #[local]
    struct Local {}

    #[init(local = [wait_queue: WaitQueue = WaitQueue::new()])]
    fn init(cx: init::Context) -> (Shared, Local) {
        Mono::start(cx.core.SYST, 12_000_000);

        incrementer::spawn(cx.local.wait_queue).ok().unwrap();
        waiter::spawn(cx.local.wait_queue).ok().unwrap();

        let count = 0;

        (Shared { count }, Local {})
    }

    #[task(shared = [count])]
    async fn incrementer(mut c: incrementer::Context, wait_queue: &'static WaitQueue) {
        loop {
            hprintln!("Inc");

            c.shared.count.lock(|c| *c += 1);

            while let Some(waker) = wait_queue.pop() {
                waker.wake();
            }

            Mono::delay(10.millis()).await;
        }
    }

    #[task(shared = [count])]
    async fn waiter(mut c: waiter::Context, wait_queue: &'static WaitQueue) {
        let value = wait_queue
            .wait_until(|| c.shared.count.lock(|c| (*c >= 3).then_some(*c)))
            .await;

        hprintln!("Got {}", value);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
