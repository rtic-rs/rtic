#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;
use systick_monotonic::*;

// NOTES:
//
// - Async tasks cannot have `#[lock_free]` resources, as they can interleve and each async
//   task can have a mutable reference stored.
// - Spawning an async task equates to it being polled at least once.
// - ...

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use crate::*;

    pub type AppInstant = <Systick<100> as rtic::Monotonic>::Instant;
    pub type AppDuration = <Systick<100> as rtic::Monotonic>::Duration;

    #[shared]
    struct Shared {
        s: u32,
    }

    #[local]
    struct Local {}

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("init").unwrap();

        (
            Shared { s: 0 },
            Local {},
            init::Monotonics(Systick::new(cx.core.SYST, 12_000_000)),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // debug::exit(debug::EXIT_SUCCESS);
        loop {
            // hprintln!("idle");
            cortex_m::asm::wfi(); // put the MCU in sleep mode until interrupt occurs
        }
    }

    #[task(priority = 2)]
    async fn task(cx: task::Context) {
        hprintln!("delay long time").ok();

        let fut = Delay::spawn(2500.millis());

        hprintln!("we have just created the future").ok();
        fut.await;
        hprintln!("long delay done").ok();

        hprintln!("delay short time").ok();
        delay(500.millis()).await;
        hprintln!("short delay done").ok();

        hprintln!("test timeout").ok();
        let res = timeout(NeverEndingFuture {}, 1.secs()).await;
        hprintln!("timeout done: {:?}", res).ok();

        hprintln!("test timeout 2").ok();
        let res = timeout(delay(500.millis()), 1.secs()).await;
        hprintln!("timeout done 2: {:?}", res).ok();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(capacity = 12)]
    fn delay_handler(_: delay_handler::Context, waker: Waker) {
        waker.wake();
    }
}

// Delay

pub struct Delay {
    until: crate::app::AppInstant,
}

impl Delay {
    pub fn spawn(duration: crate::app::AppDuration) -> Self {
        let until = crate::app::monotonics::now() + duration;

        Delay { until }
    }
}

#[inline(always)]
pub fn delay(duration: crate::app::AppDuration) -> Delay {
    Delay::spawn(duration)
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.as_mut();
        let now = crate::app::monotonics::now();

        hprintln!("    poll Delay").ok();

        if now >= s.until {
            Poll::Ready(())
        } else {
            let waker = cx.waker().clone();
            crate::app::delay_handler::spawn_after(s.until - now, waker).ok();

            Poll::Pending
        }
    }
}

//=============
// Timeout future

#[derive(Copy, Clone, Debug)]
pub struct TimeoutError;

pub struct Timeout<F: Future> {
    future: F,
    until: crate::app::AppInstant,
    cancel_handle: Option<crate::app::delay_handler::SpawnHandle>,
}

impl<F> Timeout<F>
where
    F: Future,
{
    pub fn timeout(future: F, duration: crate::app::AppDuration) -> Self {
        let until = crate::app::monotonics::now() + duration;
        Self {
            future,
            until,
            cancel_handle: None,
        }
    }
}

#[inline(always)]
pub fn timeout<F: Future>(future: F, duration: crate::app::AppDuration) -> Timeout<F> {
    Timeout::timeout(future, duration)
}

impl<F> Future for Timeout<F>
where
    F: Future,
{
    type Output = Result<F::Output, TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = crate::app::monotonics::now();

        // SAFETY: We don't move the underlying pinned value.
        let mut s = unsafe { self.get_unchecked_mut() };
        let future = unsafe { Pin::new_unchecked(&mut s.future) };

        hprintln!("    poll Timeout").ok();

        match future.poll(cx) {
            Poll::Ready(r) => {
                if let Some(ch) = s.cancel_handle.take() {
                    ch.cancel().ok();
                }

                Poll::Ready(Ok(r))
            }
            Poll::Pending => {
                if now >= s.until {
                    Poll::Ready(Err(TimeoutError))
                } else if s.cancel_handle.is_none() {
                    let waker = cx.waker().clone();
                    let sh = crate::app::delay_handler::spawn_after(s.until - now, waker)
                        .expect("Internal RTIC bug, this should never fail");
                    s.cancel_handle = Some(sh);

                    Poll::Pending
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

pub struct NeverEndingFuture {}

impl Future for NeverEndingFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        // Never finish
        hprintln!("    polling NeverEndingFuture").ok();
        Poll::Pending
    }
}
