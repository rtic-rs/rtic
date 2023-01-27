//! Crate

#![no_std]
#![deny(missing_docs)]
//deny_warnings_placeholder_for_ci

use core::{
    cell::UnsafeCell,
    future::poll_fn,
    mem::MaybeUninit,
    ptr,
    task::{Poll, Waker},
};
use heapless::Deque;
use wait_queue::WaitQueue;
use waker_registration::CriticalSectionWakerRegistration as WakerRegistration;

mod wait_queue;
mod waker_registration;

/// An MPSC channel for use in no-alloc systems. `N` sets the size of the queue.
///
/// This channel uses critical sections, however there are extremely small and all `memcpy`
/// operations of `T` are done without critical sections.
pub struct Channel<T, const N: usize> {
    // Here are all indexes that are not used in `slots` and ready to be allocated.
    freeq: UnsafeCell<Deque<u8, N>>,
    // Here are wakers and indexes to slots that are ready to be dequeued by the receiver.
    readyq: UnsafeCell<Deque<u8, N>>,
    // Waker for the receiver.
    receiver_waker: WakerRegistration,
    // Storage for N `T`s, so we don't memcpy around a lot of `T`s.
    slots: [UnsafeCell<MaybeUninit<T>>; N],
    // If there is no room in the queue a `Sender`s can wait for there to be place in the queue.
    wait_queue: WaitQueue,
    // Keep track of the receiver.
    receiver_dropped: UnsafeCell<bool>,
    // Keep track of the number of senders.
    num_senders: UnsafeCell<usize>,
}

struct UnsafeAccess<'a, const N: usize> {
    freeq: &'a mut Deque<u8, N>,
    readyq: &'a mut Deque<u8, N>,
    receiver_dropped: &'a mut bool,
    num_senders: &'a mut usize,
}

impl<T, const N: usize> Channel<T, N> {
    const _CHECK: () = assert!(N < 256, "This queue support a maximum of 255 entries");

    const INIT_SLOTS: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    /// Create a new channel.
    pub const fn new() -> Self {
        Self {
            freeq: UnsafeCell::new(Deque::new()),
            readyq: UnsafeCell::new(Deque::new()),
            receiver_waker: WakerRegistration::new(),
            slots: [Self::INIT_SLOTS; N],
            wait_queue: WaitQueue::new(),
            receiver_dropped: UnsafeCell::new(false),
            num_senders: UnsafeCell::new(0),
        }
    }

    /// Split the queue into a `Sender`/`Receiver` pair.
    pub fn split<'a>(&'a mut self) -> (Sender<'a, T, N>, Receiver<'a, T, N>) {
        // Fill free queue
        for idx in 0..(N - 1) as u8 {
            debug_assert!(!self.freeq.get_mut().is_full());

            // SAFETY: This safe as the loop goes from 0 to the capacity of the underlying queue.
            unsafe {
                self.freeq.get_mut().push_back_unchecked(idx);
            }
        }

        debug_assert!(self.freeq.get_mut().is_full());

        // There is now 1 sender
        *self.num_senders.get_mut() = 1;

        (Sender(self), Receiver(self))
    }

    fn access<'a>(&'a self, _cs: critical_section::CriticalSection) -> UnsafeAccess<'a, N> {
        // SAFETY: This is safe as are in a critical section.
        unsafe {
            UnsafeAccess {
                freeq: &mut *self.freeq.get(),
                readyq: &mut *self.readyq.get(),
                receiver_dropped: &mut *self.receiver_dropped.get(),
                num_senders: &mut *self.num_senders.get(),
            }
        }
    }
}

/// Creates a split channel with `'static` lifetime.
#[macro_export]
macro_rules! make_channel {
    ($type:path, $size:expr) => {{
        static mut CHANNEL: Channel<$type, $size> = Channel::new();

        // SAFETY: This is safe as we hide the static mut from others to access it.
        // Only this point is where the mutable access happens.
        unsafe { CHANNEL.split() }
    }};
}

// -------- Sender

/// Error state for when the receiver has been dropped.
pub struct NoReceiver<T>(pub T);

/// A `Sender` can send to the channel and can be cloned.
pub struct Sender<'a, T, const N: usize>(&'a Channel<T, N>);

unsafe impl<'a, T, const N: usize> Send for Sender<'a, T, N> {}

impl<'a, T, const N: usize> Sender<'a, T, N> {
    #[inline(always)]
    fn send_footer(&mut self, idx: u8, val: T) {
        // Write the value to the slots, note; this memcpy is not under a critical section.
        unsafe {
            ptr::write(
                self.0.slots.get_unchecked(idx as usize).get() as *mut T,
                val,
            )
        }

        // Write the value into the ready queue.
        critical_section::with(|cs| unsafe { self.0.access(cs).readyq.push_back_unchecked(idx) });

        // If there is a receiver waker, wake it.
        self.0.receiver_waker.wake();
    }

    /// Try to send a value, non-blocking. If the channel is full this will return an error.
    /// Note; this does not check if the channel is closed.
    pub fn try_send(&mut self, val: T) -> Result<(), T> {
        // If the wait queue is not empty, we can't try to push into the queue.
        if !self.0.wait_queue.is_empty() {
            return Err(val);
        }

        let idx =
            if let Some(idx) = critical_section::with(|cs| self.0.access(cs).freeq.pop_front()) {
                idx
            } else {
                return Err(val);
            };

        self.send_footer(idx, val);

        Ok(())
    }

    /// Send a value. If there is no place left in the queue this will wait until there is.
    /// If the receiver does not exist this will return an error.
    pub async fn send(&mut self, val: T) -> Result<(), NoReceiver<T>> {
        if self.is_closed() {}

        let mut __hidden_link: Option<wait_queue::Link<Waker>> = None;

        // Make this future `Drop`-safe
        let link_ptr = &mut __hidden_link as *mut Option<wait_queue::Link<Waker>>;
        let dropper = OnDrop::new(|| {
            // SAFETY: We only run this closure and dereference the pointer if we have
            // exited the `poll_fn` below in the `drop(dropper)` call. The other dereference
            // of this pointer is in the `poll_fn`.
            if let Some(link) = unsafe { &mut *link_ptr } {
                link.remove_from_list(&self.0.wait_queue);
            }
        });

        let idx = poll_fn(|cx| {
            if self.is_closed() {
                return Poll::Ready(Err(()));
            }

            //  Do all this in one critical section, else there can be race conditions
            let queue_idx = critical_section::with(|cs| {
                if !self.0.wait_queue.is_empty() || self.0.access(cs).freeq.is_empty() {
                    // SAFETY: This pointer is only dereferenced here and on drop of the future.
                    let link = unsafe { &mut *link_ptr };
                    if link.is_none() {
                        // Place the link in the wait queue on first run.
                        let link_ref = link.insert(wait_queue::Link::new(cx.waker().clone()));
                        self.0.wait_queue.push(link_ref);
                    }

                    return None;
                }

                // Get index as the queue is guaranteed not empty and the wait queue is empty
                let idx = unsafe { self.0.access(cs).freeq.pop_front_unchecked() };

                Some(idx)
            });

            if let Some(idx) = queue_idx {
                // Return the index
                Poll::Ready(Ok(idx))
            } else {
                return Poll::Pending;
            }
        })
        .await;

        // Make sure the link is removed from the queue.
        drop(dropper);

        if let Ok(idx) = idx {
            self.send_footer(idx, val);

            Ok(())
        } else {
            Err(NoReceiver(val))
        }
    }

    /// Returns true if there is no `Receiver`s.
    pub fn is_closed(&self) -> bool {
        critical_section::with(|cs| *self.0.access(cs).receiver_dropped)
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        critical_section::with(|cs| self.0.access(cs).freeq.is_empty())
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        critical_section::with(|cs| self.0.access(cs).freeq.is_full())
    }
}

impl<'a, T, const N: usize> Drop for Sender<'a, T, N> {
    fn drop(&mut self) {
        // Count down the reference counter
        let num_senders = critical_section::with(|cs| {
            *self.0.access(cs).num_senders -= 1;

            *self.0.access(cs).num_senders
        });

        // If there are no senders, wake the receiver to do error handling.
        if num_senders == 0 {
            self.0.receiver_waker.wake();
        }
    }
}

impl<'a, T, const N: usize> Clone for Sender<'a, T, N> {
    fn clone(&self) -> Self {
        // Count up the reference counter
        critical_section::with(|cs| *self.0.access(cs).num_senders += 1);

        Self(self.0)
    }
}

// -------- Receiver

/// A receiver of the channel. There can only be one receiver at any time.
pub struct Receiver<'a, T, const N: usize>(&'a Channel<T, N>);

/// Error state for when all senders has been dropped.
pub struct NoSender;

impl<'a, T, const N: usize> Receiver<'a, T, N> {
    /// Receives a value if there is one in the channel, non-blocking.
    /// Note; this does not check if the channel is closed.
    pub fn try_recv(&mut self) -> Option<T> {
        // Try to get a ready slot.
        let ready_slot =
            critical_section::with(|cs| self.0.access(cs).readyq.pop_front().map(|rs| rs));

        if let Some(rs) = ready_slot {
            // Read the value from the slots, note; this memcpy is not under a critical section.
            let r = unsafe { ptr::read(self.0.slots.get_unchecked(rs as usize).get() as *const T) };

            // Return the index to the free queue after we've read the value.
            critical_section::with(|cs| unsafe { self.0.access(cs).freeq.push_back_unchecked(rs) });

            // If someone is waiting in the WaiterQueue, wake the first one up.
            if let Some(wait_head) = self.0.wait_queue.pop() {
                wait_head.wake();
            }

            Some(r)
        } else {
            None
        }
    }

    /// Receives a value, waiting if the queue is empty.
    /// If all senders are dropped this will error with `NoSender`.
    pub async fn recv(&mut self) -> Result<T, NoSender> {
        // There was nothing in the queue, setup the waiting.
        poll_fn(|cx| {
            // Register waker.
            // TODO: Should it happen here or after the if? This might cause a spurious wake.
            self.0.receiver_waker.register(cx.waker());

            // Try to dequeue.
            if let Some(val) = self.try_recv() {
                return Poll::Ready(Ok(val));
            }

            // If the queue is empty and there is no sender, return the error.
            if self.is_closed() {
                return Poll::Ready(Err(NoSender));
            }

            Poll::Pending
        })
        .await
    }

    /// Returns true if there are no `Sender`s.
    pub fn is_closed(&self) -> bool {
        critical_section::with(|cs| *self.0.access(cs).num_senders == 0)
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        critical_section::with(|cs| self.0.access(cs).readyq.is_empty())
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        critical_section::with(|cs| self.0.access(cs).readyq.is_empty())
    }
}

impl<'a, T, const N: usize> Drop for Receiver<'a, T, N> {
    fn drop(&mut self) {
        // Mark the receiver as dropped and wake all waiters
        critical_section::with(|cs| *self.0.access(cs).receiver_dropped = true);

        while let Some(waker) = self.0.wait_queue.pop() {
            waker.wake();
        }
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

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    #[test]
    fn channel() {}
}
