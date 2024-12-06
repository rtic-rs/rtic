//! A generic timer queue for async executors.

use crate::linked_list::{self, Link, LinkedList};
use crate::TimeoutError;

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::task::{Poll, Waker};

mod backend;
mod tick_type;
pub use backend::TimerQueueBackend;
pub use tick_type::TimerQueueTicks;

/// Holds a waker and at which time instant this waker shall be awoken.
struct WaitingWaker<Backend: TimerQueueBackend> {
    waker: Waker,
    release_at: Backend::Ticks,
    was_popped: AtomicBool,
}

impl<Backend: TimerQueueBackend> Clone for WaitingWaker<Backend> {
    fn clone(&self) -> Self {
        Self {
            waker: self.waker.clone(),
            release_at: self.release_at,
            was_popped: AtomicBool::new(self.was_popped.load(Ordering::Relaxed)),
        }
    }
}

impl<Backend: TimerQueueBackend> PartialEq for WaitingWaker<Backend> {
    fn eq(&self, other: &Self) -> bool {
        self.release_at == other.release_at
    }
}

impl<Backend: TimerQueueBackend> PartialOrd for WaitingWaker<Backend> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.release_at.compare(other.release_at))
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
/// This timer queue is based on an intrusive linked list, and by extension the links are stored
/// on the async stacks of callers. The links are deallocated on `drop` or when the wait is
/// complete.
///
/// Do not call `mem::forget` on an awaited future, or there will be dragons!
pub struct TimerQueue<Backend: TimerQueueBackend> {
    queue: LinkedList<WaitingWaker<Backend>>,
    initialized: AtomicBool,
}

impl<Backend: TimerQueueBackend> Default for TimerQueue<Backend> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Backend: TimerQueueBackend> TimerQueue<Backend> {
    /// Make a new queue.
    pub const fn new() -> Self {
        Self {
            queue: LinkedList::new(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Forwards the `Monotonic::now()` method.
    #[inline(always)]
    pub fn now(&self) -> Backend::Ticks {
        Backend::now()
    }

    /// Takes the initialized monotonic to initialize the TimerQueue.
    pub fn initialize(&self, backend: Backend) {
        self.initialized.store(true, Ordering::SeqCst);

        // Don't run drop on `Backend`
        core::mem::forget(backend);
    }

    /// Call this in the interrupt handler of the hardware timer supporting the `Monotonic`
    ///
    /// # Safety
    ///
    /// It's always safe to call, but it must only be called from the interrupt of the
    /// monotonic timer for correct operation.
    pub unsafe fn on_monotonic_interrupt(&self) {
        Backend::clear_compare_flag();
        Backend::on_interrupt();

        loop {
            let mut release_at = None;
            let head = self.queue.pop_if(|head| {
                release_at = Some(head.release_at);

                let should_pop = Backend::now().is_at_least(head.release_at);
                head.was_popped.store(should_pop, Ordering::Relaxed);

                should_pop
            });

            match (head, release_at) {
                (Some(link), _) => {
                    link.waker.wake();
                }
                (None, Some(instant)) => {
                    Backend::enable_timer();
                    Backend::set_compare(instant);

                    if Backend::now().is_at_least(instant) {
                        // The time for the next instant passed while handling it,
                        // continue dequeueing
                        continue;
                    }

                    break;
                }
                (None, None) => {
                    // Queue is empty
                    Backend::disable_timer();

                    break;
                }
            }
        }
    }

    /// Timeout at a specific time.
    pub fn timeout_at<F: Future>(
        &self,
        instant: Backend::Ticks,
        future: F,
    ) -> Timeout<'_, Backend, F> {
        Timeout {
            delay: Delay::<Backend> {
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
        duration: Backend::Ticks,
        future: F,
    ) -> Timeout<'_, Backend, F> {
        let now = Backend::now();
        let mut timeout = now.wrapping_add(duration);
        if now != timeout {
            timeout = timeout.wrapping_add(Backend::Ticks::ONE_TICK);
        }

        // Wait for one period longer, because by definition timers have an uncertainty
        // of one period, so waiting for 'at least' needs to compensate for that.
        self.timeout_at(timeout, future)
    }

    /// Delay for at least some duration of time.
    #[inline]
    pub fn delay(&self, duration: Backend::Ticks) -> Delay<'_, Backend> {
        let now = Backend::now();
        let mut timeout = now.wrapping_add(duration);
        if now != timeout {
            timeout = timeout.wrapping_add(Backend::Ticks::ONE_TICK);
        }

        // Wait for one period longer, because by definition timers have an uncertainty
        // of one period, so waiting for 'at least' needs to compensate for that.
        self.delay_until(timeout)
    }

    /// Delay to some specific time instant.
    pub fn delay_until(&self, instant: Backend::Ticks) -> Delay<'_, Backend> {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!(
                "The timer queue is not initialized with a monotonic, you need to run `initialize`"
            );
        }
        Delay::<Backend> {
            instant,
            queue: &self.queue,
            link_ptr: None,
            marker: AtomicUsize::new(0),
        }
    }
}

/// Future returned by `delay` and `delay_until`.
pub struct Delay<'q, Backend: TimerQueueBackend> {
    instant: Backend::Ticks,
    queue: &'q LinkedList<WaitingWaker<Backend>>,
    link_ptr: Option<linked_list::Link<WaitingWaker<Backend>>>,
    marker: AtomicUsize,
}

impl<Backend: TimerQueueBackend> Future for Delay<'_, Backend> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We ensure we never move anything out of this.
        let this = unsafe { self.get_unchecked_mut() };

        if Backend::now().is_at_least(this.instant) {
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
                Backend::pend_interrupt()
            }
        }

        Poll::Pending
    }
}

impl<Backend: TimerQueueBackend> Drop for Delay<'_, Backend> {
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
pub struct Timeout<'q, Backend: TimerQueueBackend, F> {
    delay: Delay<'q, Backend>,
    future: F,
}

impl<Backend: TimerQueueBackend, F: Future> Future for Timeout<'_, Backend, F> {
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
