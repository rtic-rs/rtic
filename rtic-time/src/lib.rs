//! Time-related traits & structs.
//!
//! This crate contains basic definitions and utilities that can be used
//! to keep track of time.

#![no_std]
#![deny(missing_docs)]
#![allow(incomplete_features)]

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::task::{Poll, Waker};
use linked_list::{Link, LinkedList};
pub use monotonic::Monotonic;

pub mod half_period_counter;
mod linked_list;
mod monotonic;

/// Holds a waker and at which time instant this waker shall be awoken.
struct WaitingWaker<Mono: Monotonic> {
    waker: Waker,
    release_at: Mono::Instant,
    was_popped: AtomicBool,
}

impl<Mono: Monotonic> Clone for WaitingWaker<Mono> {
    fn clone(&self) -> Self {
        Self {
            waker: self.waker.clone(),
            release_at: self.release_at,
            was_popped: AtomicBool::new(self.was_popped.load(Ordering::Relaxed)),
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

                let should_pop = Mono::now() >= head.release_at;
                head.was_popped.store(should_pop, Ordering::Relaxed);

                should_pop
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
    pub fn timeout_at<F: Future>(&self, instant: Mono::Instant, future: F) -> Timeout<'_, Mono, F> {
        Timeout {
            delay: Delay::<Mono> {
                instant,
                queue: &self.queue,
                link_ptr: None,
                marker: AtomicUsize::new(0),
            },
            future,
        }
    }

    /// Timeout after at least a specific duration.
    #[inline]
    pub fn timeout_after<F: Future>(
        &self,
        duration: Mono::Duration,
        future: F,
    ) -> Timeout<'_, Mono, F> {
        let now = Mono::now();
        let mut timeout = now + duration;
        if now != timeout {
            timeout = timeout + Mono::TICK_PERIOD;
        }

        // Wait for one period longer, because by definition timers have an uncertainty
        // of one period, so waiting for 'at least' needs to compensate for that.
        self.timeout_at(timeout, future)
    }

    /// Delay for at least some duration of time.
    #[inline]
    pub fn delay(&self, duration: Mono::Duration) -> Delay<'_, Mono> {
        let now = Mono::now();
        let mut timeout = now + duration;
        if now != timeout {
            timeout = timeout + Mono::TICK_PERIOD;
        }

        // Wait for one period longer, because by definition timers have an uncertainty
        // of one period, so waiting for 'at least' needs to compensate for that.
        self.delay_until(timeout)
    }

    /// Delay to some specific time instant.
    pub fn delay_until(&self, instant: Mono::Instant) -> Delay<'_, Mono> {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!(
                "The timer queue is not initialized with a monotonic, you need to run `initialize`"
            );
        }
        Delay::<Mono> {
            instant,
            queue: &self.queue,
            link_ptr: None,
            marker: AtomicUsize::new(0),
        }
    }
}

/// Future returned by `delay` and `delay_until`.
pub struct Delay<'q, Mono: Monotonic> {
    instant: Mono::Instant,
    queue: &'q LinkedList<WaitingWaker<Mono>>,
    link_ptr: Option<linked_list::Link<WaitingWaker<Mono>>>,
    marker: AtomicUsize,
}

impl<'q, Mono: Monotonic> Future for Delay<'q, Mono> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We ensure we never move anything out of this.
        let this = unsafe { self.get_unchecked_mut() };

        if Mono::now() >= this.instant {
            return Poll::Ready(());
        }

        // SAFETY: this is dereferenced only here and in `drop`. As the queue deletion is done only
        // in `drop` we can't do this access concurrently with queue removal.
        let link = &mut this.link_ptr;
        if link.is_none() {
            let link_ref = link.insert(Link::new(WaitingWaker {
                waker: cx.waker().clone(),
                release_at: this.instant,
                was_popped: AtomicBool::new(false),
            }));

            // SAFETY(new_unchecked): The address to the link is stable as it is defined
            // outside this stack frame.
            // SAFETY(insert): `link_ref` lfetime comes from `link_ptr` which itself is owned by
            // the `Delay` struct. The `Delay::drop` impl ensures that the link is removed from the
            // queue on drop, which happens before the struct and thus `link_ptr` goes out of
            // scope.
            let (head_updated, addr) = unsafe { this.queue.insert(Pin::new_unchecked(link_ref)) };
            this.marker.store(addr, Ordering::Relaxed);
            if head_updated {
                Mono::pend_interrupt()
            }
        }

        Poll::Pending
    }
}

impl<'q, Mono: Monotonic> Drop for Delay<'q, Mono> {
    fn drop(&mut self) {
        // SAFETY: Drop cannot be run at the same time as poll, so we can't end up
        // derefencing this concurrently to the one in `poll`.
        match self.link_ptr.as_ref() {
            None => return,
            // If it was popped from the queue there is no need to run delete
            Some(link) if link.val.was_popped.load(Ordering::Relaxed) => return,
            _ => {}
        }
        self.queue.delete(self.marker.load(Ordering::Relaxed));
    }
}

/// Future returned by `timeout` and `timeout_at`.
pub struct Timeout<'q, Mono: Monotonic, F> {
    delay: Delay<'q, Mono>,
    future: F,
}

impl<'q, Mono: Monotonic, F: Future> Future for Timeout<'q, Mono, F> {
    type Output = Result<F::Output, TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let inner = unsafe { self.get_unchecked_mut() };

        {
            let f = unsafe { Pin::new_unchecked(&mut inner.future) };
            if let Poll::Ready(v) = f.poll(cx) {
                return Poll::Ready(Ok(v));
            }
        }

        {
            let d = unsafe { Pin::new_unchecked(&mut inner.delay) };
            if d.poll(cx).is_ready() {
                return Poll::Ready(Err(TimeoutError));
            }
        }

        Poll::Pending
    }
}
