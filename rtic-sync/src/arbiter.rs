//! A Mutex-like FIFO with unlimited-waiter for embedded systems.
//!
//! Example usage:
//!
//! ```rust
//! # async fn select<F1, F2>(f1: F1, f2: F2) {}
//! use rtic_sync::arbiter::Arbiter;
//!
//! // Instantiate an Arbiter with a static lifetime.
//! static ARBITER: Arbiter<u32> = Arbiter::new(32);
//!
//! async fn run(){
//!     let write_42 = async move {
//!         *ARBITER.access().await = 42;
//!     };
//!
//!     let write_1337 = async move {
//!         *ARBITER.access().await = 1337;
//!     };
//!
//!     // Attempt to access the Arbiter concurrently.
//!     select(write_42, write_1337).await;
//! }
//! ```

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::sync::atomic::{fence, AtomicBool, Ordering};
use core::task::{Poll, Waker};

use rtic_common::dropper::OnDrop;
use rtic_common::wait_queue::{Link, WaitQueue};

/// This is needed to make the async closure in `send` accept that we "share"
/// the link possible between threads.
#[derive(Clone)]
struct LinkPtr(*mut Option<Link<Waker>>);

impl LinkPtr {
    /// This will dereference the pointer stored within and give out an `&mut`.
    unsafe fn get(&mut self) -> &mut Option<Link<Waker>> {
        &mut *self.0
    }
}

unsafe impl Send for LinkPtr {}
unsafe impl Sync for LinkPtr {}

/// An FIFO waitqueue for use in shared bus usecases.
pub struct Arbiter<T> {
    wait_queue: WaitQueue,
    inner: UnsafeCell<T>,
    taken: AtomicBool,
}

unsafe impl<T> Send for Arbiter<T> {}
unsafe impl<T> Sync for Arbiter<T> {}

impl<T> Arbiter<T> {
    /// Create a new arbiter.
    pub const fn new(inner: T) -> Self {
        Self {
            wait_queue: WaitQueue::new(),
            inner: UnsafeCell::new(inner),
            taken: AtomicBool::new(false),
        }
    }

    /// Get access to the inner value in the [`Arbiter`]. This will wait until access is granted,
    /// for non-blocking access use `try_access`.
    pub async fn access(&self) -> ExclusiveAccess<'_, T> {
        let mut link_ptr: Option<Link<Waker>> = None;

        // Make this future `Drop`-safe.
        // SAFETY(link_ptr): Shadow the original definition of `link_ptr` so we can't abuse it.
        let mut link_ptr = LinkPtr(&mut link_ptr as *mut Option<Link<Waker>>);

        let mut link_ptr2 = link_ptr.clone();
        let dropper = OnDrop::new(|| {
            // SAFETY: We only run this closure and dereference the pointer if we have
            // exited the `poll_fn` below in the `drop(dropper)` call. The other dereference
            // of this pointer is in the `poll_fn`.
            if let Some(link) = unsafe { link_ptr2.get() } {
                link.remove_from_list(&self.wait_queue);
            }
        });

        poll_fn(|cx| {
            critical_section::with(|_| {
                fence(Ordering::SeqCst);

                // The queue is empty and noone has taken the value.
                if self.wait_queue.is_empty() && !self.taken.load(Ordering::Relaxed) {
                    self.taken.store(true, Ordering::Relaxed);

                    return Poll::Ready(());
                }

                // SAFETY: This pointer is only dereferenced here and on drop of the future
                // which happens outside this `poll_fn`'s stack frame.
                let link = unsafe { link_ptr.get() };
                if let Some(link) = link {
                    if link.is_popped() {
                        return Poll::Ready(());
                    }
                } else {
                    // Place the link in the wait queue on first run.
                    let link_ref = link.insert(Link::new(cx.waker().clone()));

                    // SAFETY(new_unchecked): The address to the link is stable as it is defined
                    // outside this stack frame.
                    // SAFETY(push): `link_ref` lifetime comes from `link_ptr` that is shadowed,
                    // and  we make sure in `dropper` that the link is removed from the queue
                    // before dropping `link_ptr` AND `dropper` makes sure that the shadowed
                    // `link_ptr` lives until the end of the stack frame.
                    unsafe { self.wait_queue.push(Pin::new_unchecked(link_ref)) };
                }

                Poll::Pending
            })
        })
        .await;

        // Make sure the link is removed from the queue.
        drop(dropper);

        // SAFETY: One only gets here if there is exlusive access.
        ExclusiveAccess {
            arbiter: self,
            inner: unsafe { &mut *self.inner.get() },
        }
    }

    /// Non-blockingly tries to access the underlying value.
    /// If someone is in queue to get it, this will return `None`.
    pub fn try_access(&self) -> Option<ExclusiveAccess<'_, T>> {
        critical_section::with(|_| {
            fence(Ordering::SeqCst);

            // The queue is empty and noone has taken the value.
            if self.wait_queue.is_empty() && !self.taken.load(Ordering::Relaxed) {
                self.taken.store(true, Ordering::Relaxed);

                // SAFETY: One only gets here if there is exlusive access.
                Some(ExclusiveAccess {
                    arbiter: self,
                    inner: unsafe { &mut *self.inner.get() },
                })
            } else {
                None
            }
        })
    }
}

/// This token represents exclusive access to the value protected by the [`Arbiter`].
pub struct ExclusiveAccess<'a, T> {
    arbiter: &'a Arbiter<T>,
    inner: &'a mut T,
}

impl<'a, T> Drop for ExclusiveAccess<'a, T> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            fence(Ordering::SeqCst);

            if self.arbiter.wait_queue.is_empty() {
                // If noone is in queue and we release exclusive access, reset `taken`.
                self.arbiter.taken.store(false, Ordering::Relaxed);
            } else if let Some(next) = self.arbiter.wait_queue.pop() {
                // Wake the next one in queue.
                next.wake();
            }
        })
    }
}

impl<'a, T> Deref for ExclusiveAccess<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for ExclusiveAccess<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stress_channel() {
        const NUM_RUNS: usize = 100_000;

        static ARB: Arbiter<usize> = Arbiter::new(0);
        let mut v = std::vec::Vec::new();

        for _ in 0..NUM_RUNS {
            v.push(tokio::spawn(async move {
                *ARB.access().await += 1;
            }));
        }

        for v in v {
            v.await.unwrap();
        }

        assert_eq!(*ARB.access().await, NUM_RUNS)
    }
}
