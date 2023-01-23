#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

// NOTES:
//
// - Async tasks cannot have `#[lock_free]` resources, as they can interleve and each async
//   task can have a mutable reference stored.
// - Spawning an async task equates to it being polled once.

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };
    use cortex_m_semihosting::{debug, hprintln};
    use systick_monotonic::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("init").unwrap();

        foo::spawn().ok();
        bar::spawn().ok();

        (
            Shared {},
            Local {},
            init::Monotonics(Systick::new(cx.core.SYST, 12_000_000)),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi(); // put the MCU in sleep mode until interrupt occurs
        }
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        hprintln!("hello from foo").ok();

        // This will not timeout
        match monotonics::timeout_after(monotonics::delay(100.millis()), 200.millis()).await {
            Ok(_) => hprintln!("foo no timeout").ok(),
            Err(_) => hprintln!("foo timeout").ok(),
        };
    }

    #[task]
    async fn bar(_cx: bar::Context) {
        hprintln!("hello from bar").ok();

        // This will timeout
        match monotonics::timeout_after(NeverEndingFuture {}, 300.millis()).await {
            Ok(_) => hprintln!("bar no timeout").ok(),
            Err(_) => hprintln!("bar timeout").ok(),
        };

        debug::exit(debug::EXIT_SUCCESS);
    }

    pub struct NeverEndingFuture {}

    impl Future for NeverEndingFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            // Never finish
            Poll::Pending
        }
    }
}
