//! A generic timer queue for async executors.

use crate::linked_list::{self, Link, LinkedList};
use crate::TimeoutError;

use core::future::{poll_fn, Future};
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::task::{Poll, Waker};
use futures_util::{
    future::{select, Either},
    pin_mut,
};
use rtic_common::dropper::OnDrop;

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

/// This is needed to make the async closure in `delay_until` accept that we "share"
/// the link possible between threads.
struct LinkPtr<Backend: TimerQueueBackend>(*mut Option<linked_list::Link<WaitingWaker<Backend>>>);

impl<Backend: TimerQueueBackend> Clone for LinkPtr<Backend> {
    fn clone(&self) -> Self {
        LinkPtr(self.0)
    }
}

impl<Backend: TimerQueueBackend> LinkPtr<Backend> {
    /// This will dereference the pointer stored within and give out an `&mut`.
    unsafe fn get(&mut self) -> &mut Option<linked_list::Link<WaitingWaker<Backend>>> {
        &mut *self.0
    }
}

unsafe impl<Backend: TimerQueueBackend> Send for LinkPtr<Backend> {}
unsafe impl<Backend: TimerQueueBackend> Sync for LinkPtr<Backend> {}

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

        // Don't run drop on `Mono`
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
    pub async fn timeout_at<F: Future>(
        &self,
        instant: Backend::Ticks,
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

    /// Timeout after at least a specific duration.
    #[inline]
    pub async fn timeout_after<F: Future>(
        &self,
        duration: Backend::Ticks,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        let now = Backend::now();
        let mut timeout = now.wrapping_add(duration);
        if now != timeout {
            timeout = timeout.wrapping_add(Backend::Ticks::ONE_TICK);
        }

        // Wait for one period longer, because by definition timers have an uncertainty
        // of one period, so waiting for 'at least' needs to compensate for that.
        self.timeout_at(timeout, future).await
    }

    /// Delay for at least some duration of time.
    #[inline]
    pub async fn delay(&self, duration: Backend::Ticks) {
        let now = Backend::now();
        let mut timeout = now.wrapping_add(duration);
        if now != timeout {
            timeout = timeout.wrapping_add(Backend::Ticks::ONE_TICK);
        }

        // Wait for one period longer, because by definition timers have an uncertainty
        // of one period, so waiting for 'at least' needs to compensate for that.
        self.delay_until(timeout).await;
    }

    /// Delay to some specific time instant.
    pub async fn delay_until(&self, instant: Backend::Ticks) {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!(
                "The timer queue is not initialized with a monotonic, you need to run `initialize`"
            );
        }

        let mut link_ptr: Option<linked_list::Link<WaitingWaker<Backend>>> = None;

        // Make this future `Drop`-safe
        // SAFETY(link_ptr): Shadow the original definition of `link_ptr` so we can't abuse it.
        let mut link_ptr =
            LinkPtr(&mut link_ptr as *mut Option<linked_list::Link<WaitingWaker<Backend>>>);
        let mut link_ptr2 = link_ptr.clone();

        let queue = &self.queue;
        let marker = &AtomicUsize::new(0);

        let dropper = OnDrop::new(|| {
            queue.delete(marker.load(Ordering::Relaxed));
        });

        poll_fn(|cx| {
            if Backend::now().is_at_least(instant) {
                return Poll::Ready(());
            }

            // SAFETY: This pointer is only dereferenced here and on drop of the future
            // which happens outside this `poll_fn`'s stack frame, so this mutable access cannot
            // happen at the same time as `dropper` runs.
            let link = unsafe { link_ptr2.get() };
            if link.is_none() {
                let link_ref = link.insert(Link::new(WaitingWaker {
                    waker: cx.waker().clone(),
                    release_at: instant,
                    was_popped: AtomicBool::new(false),
                }));

                // SAFETY(new_unchecked): The address to the link is stable as it is defined
                //outside this stack frame.
                // SAFETY(insert): `link_ref` lifetime comes from `link_ptr` that is shadowed, and
                // we make sure in `dropper` that the link is removed from the queue before
                // dropping `link_ptr` AND `dropper` makes sure that the shadowed `link_ptr` lives
                // until the end of the stack frame.
                let (head_updated, addr) = unsafe { queue.insert(Pin::new_unchecked(link_ref)) };

                marker.store(addr, Ordering::Relaxed);

                if head_updated {
                    // Pend the monotonic handler if the queue head was updated.
                    Backend::pend_interrupt()
                }
            }

            Poll::Pending
        })
        .await;

        // SAFETY: We only run this and dereference the pointer if we have
        // exited the `poll_fn` below in the `drop(dropper)` call. The other dereference
        // of this pointer is in the `poll_fn`.
        if let Some(link) = unsafe { link_ptr.get() } {
            if link.val.was_popped.load(Ordering::Relaxed) {
                // If it was popped from the queue there is no need to run delete
                dropper.defuse();
            }
        } else {
            // Make sure that our link is deleted from the list before we drop this stack
            drop(dropper);
        }
    }
}
