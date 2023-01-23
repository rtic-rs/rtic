//! Crate

#![no_std]
#![no_main]
#![deny(missing_docs)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

pub mod monotonic;

use core::future::{poll_fn, Future};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::task::{Poll, Waker};
use futures_util::{
    future::{select, Either},
    pin_mut,
};
pub use monotonic::Monotonic;

mod linked_list;

use linked_list::{Link, LinkedList};

/// Holds a waker and at which time instant this waker shall be awoken.
struct WaitingWaker<Mono: Monotonic> {
    waker: Waker,
    release_at: Mono::Instant,
}

impl<Mono: Monotonic> Clone for WaitingWaker<Mono> {
    fn clone(&self) -> Self {
        Self {
            waker: self.waker.clone(),
            release_at: self.release_at,
        }
    }
}

impl<Mono: Monotonic> PartialEq for WaitingWaker<Mono> {
    fn eq(&self, other: &Self) -> bool {
        self.release_at == other.release_at
    }
}

impl<Mono: Monotonic> PartialOrd for WaitingWaker<Mono> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.release_at.partial_cmp(&other.release_at)
    }
}

/// A generic timer queue for async executors.
///
/// # Blocking
///
/// The internal priority queue uses global critical sections to manage access. This means that
/// `await`ing a delay will cause a lock of the entire system for O(n) time. In practice the lock
/// duration is ~10 clock cycles per element in the queue.
///
/// # Safety
///
/// This timer queue is based on an intrusive linked list, and by extension the links are strored
/// on the async stacks of callers. The links are deallocated on `drop` or when the wait is
/// complete.
///
/// Do not call `mem::forget` on an awaited future, or there will be dragons!
pub struct TimerQueue<Mono: Monotonic> {
    queue: LinkedList<WaitingWaker<Mono>>,
    initialized: AtomicBool,
}

/// This indicates that there was a timeout.
pub struct TimeoutError;

impl<Mono: Monotonic> TimerQueue<Mono> {
    /// Make a new queue.
    pub const fn new() -> Self {
        Self {
            queue: LinkedList::new(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Forwards the `Monotonic::now()` method.
    #[inline(always)]
    pub fn now(&self) -> Mono::Instant {
        Mono::now()
    }

    /// Takes the initialized monotonic to initialize the TimerQueue.
    pub fn initialize(&self, monotonic: Mono) {
        self.initialized.store(true, Ordering::SeqCst);

        // Don't run drop on `Mono`
        core::mem::forget(monotonic);
    }

    /// Call this in the interrupt handler of the hardware timer supporting the `Monotonic`
    ///
    /// # Safety
    ///
    /// It's always safe to call, but it must only be called from the interrupt of the
    /// monotonic timer for correct operation.
    pub unsafe fn on_monotonic_interrupt(&self) {
        Mono::clear_compare_flag();
        Mono::on_interrupt();

        loop {
            let mut release_at = None;
            let head = self.queue.pop_if(|head| {
                release_at = Some(head.release_at);

                Mono::now() >= head.release_at
            });

            match (head, release_at) {
                (Some(link), _) => {
                    link.waker.wake();
                }
                (None, Some(instant)) => {
                    Mono::enable_timer();
                    Mono::set_compare(instant);

                    if Mono::now() >= instant {
                        // The time for the next instant passed while handling it,
                        // continue dequeueing
                        continue;
                    }

                    break;
                }
                (None, None) => {
                    // Queue is empty
                    Mono::disable_timer();

                    break;
                }
            }
        }
    }

    /// Timeout at a specific time.
    pub async fn timeout_at<F: Future>(
        &self,
        instant: Mono::Instant,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        let delay = self.delay_until(instant);

        pin_mut!(future);
        pin_mut!(delay);

        match select(future, delay).await {
            Either::Left((r, _)) => Ok(r),
            Either::Right(_) => Err(TimeoutError),
        }
    }

    /// Timeout after a specific duration.
    #[inline]
    pub async fn timeout_after<F: Future>(
        &self,
        duration: Mono::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        self.timeout_at(Mono::now() + duration, future).await
    }

    /// Delay for some duration of time.
    #[inline]
    pub async fn delay(&self, duration: Mono::Duration) {
        let now = Mono::now();

        self.delay_until(now + duration).await;
    }

    /// Delay to some specific time instant.
    pub async fn delay_until(&self, instant: Mono::Instant) {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!(
                "The timer queue is not initialized with a monotonic, you need to run `initialize`"
            );
        }

        let mut first_run = true;
        let queue = &self.queue;
        let mut link = Link::new(WaitingWaker {
            waker: poll_fn(|cx| Poll::Ready(cx.waker().clone())).await,
            release_at: instant,
        });

        let marker = &AtomicUsize::new(0);

        let dropper = OnDrop::new(|| {
            queue.delete(marker.load(Ordering::Relaxed));
        });

        poll_fn(|_| {
            if Mono::now() >= instant {
                return Poll::Ready(());
            }

            if first_run {
                first_run = false;
                let (was_empty, addr) = queue.insert(&mut link);
                marker.store(addr, Ordering::Relaxed);

                if was_empty {
                    // Pend the monotonic handler if the queue was empty to setup the timer.
                    Mono::pend_interrupt();
                }
            }

            Poll::Pending
        })
        .await;

        // Make sure that our link is deleted from the list before we drop this stack
        drop(dropper);
    }
}

struct OnDrop<F: FnOnce()> {
    f: core::mem::MaybeUninit<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self {
            f: core::mem::MaybeUninit::new(f),
        }
    }

    #[allow(unused)]
    pub fn defuse(self) {
        core::mem::forget(self)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}

// -------- Test program ---------
//
//
// use systick_monotonic::{Systick, TimerQueue};
//
// // same panicking *behavior* as `panic-probe` but doesn't print a panic message
// // this prevents the panic message being printed *twice* when `defmt::panic` is invoked
// #[defmt::panic_handler]
// fn panic() -> ! {
//     cortex_m::asm::udf()
// }
//
// /// Terminates the application and makes `probe-run` exit with exit-code = 0
// pub fn exit() -> ! {
//     loop {
//         cortex_m::asm::bkpt();
//     }
// }
//
// defmt::timestamp!("{=u64:us}", {
//     let time_us: fugit::MicrosDurationU32 = MONO.now().duration_since_epoch().convert();
//
//     time_us.ticks() as u64
// });
//
// make_systick_timer_queue!(MONO, Systick<1_000>);
//
// #[rtic::app(
//     device = nrf52832_hal::pac,
//     dispatchers = [SWI0_EGU0, SWI1_EGU1, SWI2_EGU2, SWI3_EGU3, SWI4_EGU4, SWI5_EGU5],
// )]
// mod app {
//     use super::{Systick, MONO};
//     use fugit::ExtU32;
//
//     #[shared]
//     struct Shared {}
//
//     #[local]
//     struct Local {}
//
//     #[init]
//     fn init(cx: init::Context) -> (Shared, Local) {
//         defmt::println!("init");
//
//         let systick = Systick::start(cx.core.SYST, 64_000_000);
//
//         defmt::println!("initializing monotonic");
//
//         MONO.initialize(systick);
//
//         async_task::spawn().ok();
//         async_task2::spawn().ok();
//         async_task3::spawn().ok();
//
//         (Shared {}, Local {})
//     }
//
//     #[idle]
//     fn idle(_: idle::Context) -> ! {
//         defmt::println!("idle");
//
//         loop {
//             core::hint::spin_loop();
//         }
//     }
//
//     #[task]
//     async fn async_task(_: async_task::Context) {
//         loop {
//             defmt::println!("async task waiting for 1 second");
//             MONO.delay(1.secs()).await;
//         }
//     }
//
//     #[task]
//     async fn async_task2(_: async_task2::Context) {
//         loop {
//             defmt::println!("    async task 2 waiting for 0.5 second");
//             MONO.delay(500.millis()).await;
//         }
//     }
//
//     #[task]
//     async fn async_task3(_: async_task3::Context) {
//         loop {
//             defmt::println!("        async task 3 waiting for 0.2 second");
//             MONO.delay(200.millis()).await;
//         }
//     }
// }
//
