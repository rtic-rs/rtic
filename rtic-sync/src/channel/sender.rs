use core::{future::poll_fn, pin::Pin, task::Poll};

use rtic_common::{dropper::OnDrop, wait_queue::Link};

use super::{
    channel::{FreeSlot, FreeSlotPtr, WaitQueueData},
    Channel,
};

#[cfg(feature = "defmt-03")]
use crate::defmt;

/// This is needed to make the async closure in `send` accept that we "share"
/// the link possible between threads.
#[derive(Clone)]
struct LinkPtr(*mut Option<Link<WaitQueueData>>);

impl LinkPtr {
    /// This will dereference the pointer stored within and give out an `&mut`.
    unsafe fn get(&mut self) -> &mut Option<Link<WaitQueueData>> {
        &mut *self.0
    }
}

unsafe impl Send for LinkPtr {}

unsafe impl Sync for LinkPtr {}

/// Error state for when the receiver has been dropped.
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct NoReceiver<T>(pub T);

/// Errors that 'try_send` can have.
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum TrySendError<T> {
    /// Error state for when the receiver has been dropped.
    NoReceiver(T),
    /// Error state when the queue is full.
    Full(T),
}

impl<T> core::fmt::Debug for NoReceiver<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NoReceiver({:?})", self.0)
    }
}

impl<T> core::fmt::Debug for TrySendError<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TrySendError::NoReceiver(v) => write!(f, "NoReceiver({v:?})"),
            TrySendError::Full(v) => write!(f, "Full({v:?})"),
        }
    }
}

impl<T> PartialEq for TrySendError<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TrySendError::NoReceiver(v1), TrySendError::NoReceiver(v2)) => v1.eq(v2),
            (TrySendError::NoReceiver(_), TrySendError::Full(_)) => false,
            (TrySendError::Full(_), TrySendError::NoReceiver(_)) => false,
            (TrySendError::Full(v1), TrySendError::Full(v2)) => v1.eq(v2),
        }
    }
}

/// A `Sender` can send to the channel and can be cloned.
pub struct Sender<'a, T, const N: usize>(pub(crate) &'a Channel<T, N>);

unsafe impl<T, const N: usize> Send for Sender<'_, T, N> {}

impl<T, const N: usize> core::fmt::Debug for Sender<'_, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Sender")
    }
}

#[cfg(feature = "defmt-03")]
impl<T, const N: usize> defmt::Format for Sender<'_, T, N> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Sender",)
    }
}

impl<T, const N: usize> Sender<'_, T, N> {
    /// Try to send a value, non-blocking. If the channel is full this will return an error.
    pub fn try_send(&mut self, val: T) -> Result<(), TrySendError<T>> {
        // If the wait queue is not empty, we can't try to push into the queue.
        // TODO: this no longer seems necessary: freeq items are sent directly to
        // queueing `send`s.
        // if !self.0.wait_queue.is_empty() {
        //     return Err(TrySendError::Full(val));
        // }

        // No receiver available.
        if self.is_closed() {
            return Err(TrySendError::NoReceiver(val));
        }

        let idx = if let Some(idx) = self.0.pop_free_slot() {
            idx
        } else {
            return Err(TrySendError::Full(val));
        };

        unsafe { self.0.send_value(idx, val) };

        Ok(())
    }

    /// Send a value. If there is no place left in the queue this will wait until there is.
    /// If the receiver does not exist this will return an error.
    pub async fn send(&mut self, val: T) -> Result<(), NoReceiver<T>> {
        let mut free_slot_ptr: Option<FreeSlot> = None;
        let mut link_ptr: Option<Link<WaitQueueData>> = None;

        // Make this future `Drop`-safe.
        // SAFETY(link_ptr): Shadow the original definition of `link_ptr` so we can't abuse it.
        let mut link_ptr = LinkPtr(core::ptr::addr_of_mut!(link_ptr));
        // SAFETY(new): `free_slot_ptr` is alive until at least after `link_ptr` is popped.
        let mut free_slot_ptr = unsafe { FreeSlotPtr::new(core::ptr::addr_of_mut!(free_slot_ptr)) };

        let mut link_ptr2 = link_ptr.clone();
        let mut free_slot_ptr2 = free_slot_ptr.clone();
        let dropper = OnDrop::new(|| {
            // SAFETY: We only run this closure and dereference the pointer if we have
            // exited the `poll_fn` below in the `drop(dropper)` call. The other dereference
            // of this pointer is in the `poll_fn`.
            if let Some(link) = unsafe { link_ptr2.get() } {
                self.0.remove_from_wait_queue(link);
            }

            // Return our potentially-unused free slot.
            // Potentially unnecessary c-s because our link was already popped, so there
            // is no way for anything else to access the free slot ptr. Gotta think
            // about this a bit more...
            critical_section::with(|cs| {
                if let Some(freed_slot) = unsafe { free_slot_ptr2.take(cs) } {
                    // SAFETY: `freed_slot` is a free slot in our referenced channel.
                    unsafe { self.0.return_free_slot(freed_slot) };
                }
            });
        });

        let idx = poll_fn(|cx| {
            //  Do all this in one critical section, else there can be race conditions
            critical_section::with(|cs| {
                if self.is_closed() {
                    return Poll::Ready(Err(()));
                }

                // SAFETY: This pointer is only dereferenced here and on drop of the future
                // which happens outside this `poll_fn`'s stack frame.
                let link = unsafe { link_ptr.get() };

                // We are already in the wait queue.
                if let Some(link) = link {
                    if link.is_popped() {
                        // SAFETY: `free_slot_ptr` is valid for writes until the end of this future.
                        let slot = unsafe { free_slot_ptr.take(cs) };

                        // If our link is popped, then:
                        // 1. We were popped by `return_free_lot` and provided us with a slot.
                        // 2. We were popped by `drop_receiver` and it did not provide us with a slot, and the channel is closed.
                        if let Some(slot) = slot {
                            Poll::Ready(Ok(slot))
                        } else {
                            Poll::Ready(Err(()))
                        }
                    } else {
                        Poll::Pending
                    }
                }
                // A free slot is available.
                else if let Some(free_slot) = self.0.pop_free_slot() {
                    Poll::Ready(Ok(free_slot))
                }
                // We are not in the wait queue, and no free slot is available.
                else {
                    // Place the link in the wait queue.
                    let link_ref =
                        link.insert(Link::new((cx.waker().clone(), free_slot_ptr.clone())));

                    // SAFETY(new_unchecked): The address to the link is stable as it is defined
                    // outside this stack frame.
                    // SAFETY(push): `link_ref` lifetime comes from `link_ptr` and `free_slot_ptr` that
                    // are shadowed and we make sure in `dropper` that the link is removed from the queue
                    // before dropping `link_ptr` AND `dropper` makes sure that the shadowed
                    // `ptr`s live until the end of the stack frame.
                    unsafe { self.0.push_wait_queue(Pin::new_unchecked(link_ref)) };

                    Poll::Pending
                }
            })
        })
        .await;

        // Make sure the link is removed from the queue.
        drop(dropper);

        if let Ok(slot) = idx {
            // SAFETY: `slot` is provided through a `SlotPtr` or comes from `pop_free_slot`.
            unsafe { self.0.send_value(slot, val) };

            Ok(())
        } else {
            Err(NoReceiver(val))
        }
    }

    /// Returns true if there is no `Receiver`s.
    pub fn is_closed(&self) -> bool {
        self.0.receiver_dropped()
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        // SAFETY: `self.0.freeq` is not called recursively.
        unsafe { self.0.freeq(|q| q.is_empty()) }
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        // SAFETY: `self.0.freeq` is not called recursively.
        unsafe { self.0.freeq(|q| q.is_full()) }
    }
}

impl<T, const N: usize> Drop for Sender<'_, T, N> {
    fn drop(&mut self) {
        self.0.drop_sender();
    }
}

impl<T, const N: usize> Clone for Sender<'_, T, N> {
    fn clone(&self) -> Self {
        self.0.clone_sender();

        Self(self.0)
    }
}
