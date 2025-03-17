//! An async aware MPSC channel that can be used on no-alloc systems.

use crate::unsafecell::UnsafeCell;
use core::{
    future::poll_fn,
    mem::MaybeUninit,
    pin::Pin,
    ptr,
    sync::atomic::{fence, Ordering},
    task::{Poll, Waker},
};
#[doc(hidden)]
pub use critical_section;
use heapless::Deque;
use rtic_common::{
    dropper::OnDrop, wait_queue::DoublyLinkedList, wait_queue::Link,
    waker_registration::CriticalSectionWakerRegistration as WakerRegistration,
};

#[cfg(feature = "defmt-03")]
use crate::defmt;

type WaitQueueData = (Waker, SlotPtr);
type WaitQueue = DoublyLinkedList<WaitQueueData>;

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

unsafe impl<T, const N: usize> Send for Channel<T, N> {}

unsafe impl<T, const N: usize> Sync for Channel<T, N> {}

macro_rules! cs_access {
    ($name:ident, $type:ty) => {
        /// Access the value mutably.
        ///
        /// SAFETY: this function must not be called recursively within `f`.
        unsafe fn $name<F, R>(&self, _cs: critical_section::CriticalSection, f: F) -> R
        where
            F: FnOnce(&mut $type) -> R,
        {
            let v = self.$name.get_mut();
            // SAFETY: we have exclusive access due to the critical section.
            let v = unsafe { v.deref() };
            f(v)
        }
    };
}

impl<T, const N: usize> Default for Channel<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Channel<T, N> {
    const _CHECK: () = assert!(N < 256, "This queue support a maximum of 255 entries");

    /// Create a new channel.
    #[cfg(not(loom))]
    pub const fn new() -> Self {
        Self {
            freeq: UnsafeCell::new(Deque::new()),
            readyq: UnsafeCell::new(Deque::new()),
            receiver_waker: WakerRegistration::new(),
            slots: [const { UnsafeCell::new(MaybeUninit::uninit()) }; N],
            wait_queue: WaitQueue::new(),
            receiver_dropped: UnsafeCell::new(false),
            num_senders: UnsafeCell::new(0),
        }
    }

    /// Create a new channel.
    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            freeq: UnsafeCell::new(Deque::new()),
            readyq: UnsafeCell::new(Deque::new()),
            receiver_waker: WakerRegistration::new(),
            slots: core::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())),
            wait_queue: WaitQueue::new(),
            receiver_dropped: UnsafeCell::new(false),
            num_senders: UnsafeCell::new(0),
        }
    }

    /// Clear any remaining items from this `Channel`.
    pub fn clear(&mut self) {
        for _ in self.queued_items() {}
    }

    /// Return an iterator over the still-queued items, removing them
    /// from this channel.
    pub fn queued_items(&mut self) -> impl Iterator<Item = T> + '_ {
        struct Iter<'a, T, const N: usize> {
            inner: &'a mut Channel<T, N>,
        }

        impl<T, const N: usize> Iterator for Iter<'_, T, N> {
            type Item = T;

            fn next(&mut self) -> Option<Self::Item> {
                let slot = self.inner.readyq.as_mut().pop_back()?;

                let value = unsafe {
                    // SAFETY: `ready` is a valid slot.
                    let first_element = self.inner.slots.get_unchecked(slot as usize).get_mut();
                    let ptr = first_element.deref().as_ptr();
                    // SAFETY: `ptr` points to an initialized `T`.
                    core::ptr::read(ptr)
                };

                assert!(!self.inner.freeq.as_mut().is_full());
                unsafe {
                    // SAFETY: `freeq` is not ful.
                    self.inner.freeq.as_mut().push_back_unchecked(slot);
                }

                Some(value)
            }
        }

        Iter { inner: self }
    }

    /// Split the queue into a `Sender`/`Receiver` pair.
    ///
    /// # Panics
    /// This function panics if there are items in this channel while splitting.
    ///
    /// Call [`Channel::clear`] to clear all items from it, or [`Channel::queued_items`] to retrieve
    /// an iterator that yields the values.
    pub fn split(&mut self) -> (Sender<'_, T, N>, Receiver<'_, T, N>) {
        assert!(
            self.readyq.as_mut().is_empty(),
            "Cannot re-split non-empty queue. Call `Channel::clear()`."
        );

        let freeq = self.freeq.as_mut();

        freeq.clear();

        // Fill free queue
        for idx in 0..N as u8 {
            // NOTE(assert): `split`-ing does not put `freeq` into a known-empty
            // state, so `debug_assert` is not good enough.
            assert!(!freeq.is_full());

            // SAFETY: This safe as the loop goes from 0 to the capacity of the underlying queue.
            unsafe {
                freeq.push_back_unchecked(idx);
            }
        }

        debug_assert!(freeq.is_full());

        // There is now 1 sender
        *self.num_senders.as_mut() = 1;

        (Sender(self), Receiver(self))
    }

    cs_access!(freeq, Deque<u8, N>);
    cs_access!(readyq, Deque<u8, N>);
    cs_access!(receiver_dropped, bool);
    cs_access!(num_senders, usize);

    /// Return free slot `slot` to the channel.
    ///
    /// This will do one of two things:
    /// 1. If there are any waiting `send`-ers, wake the longest-waiting one and hand it `slot`.
    /// 2. else, insert `slot` into `self.freeq`.
    ///
    /// SAFETY: `slot` must be a `u8` that is obtained by dequeueing from [`Self::readyq`].
    unsafe fn return_free_slot(&self, slot: u8) {
        critical_section::with(|cs| {
            fence(Ordering::SeqCst);

            // If someone is waiting in the `wait_queue`, wake the first one up & hand it the free slot.
            if let Some((wait_head, mut freeq_slot)) = self.wait_queue.pop() {
                // SAFETY: `freeq_slot` is valid for writes: we are in a critical
                // section & the `SlotPtr` lives for at least the duration of the wait queue link.
                unsafe { freeq_slot.replace(Some(slot), cs) };
                wait_head.wake();
            } else {
                // SAFETY: `self.freeq` is not called recursively.
                unsafe {
                    self.freeq(cs, |freeq| {
                        assert!(!freeq.is_full());
                        // SAFETY: `freeq` is not full.
                        freeq.push_back_unchecked(slot);
                    });
                }
            }
        })
    }
}

impl<T, const N: usize> Drop for Channel<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Creates a split channel with `'static` lifetime.
#[macro_export]
#[cfg(not(loom))]
macro_rules! make_channel {
    ($type:ty, $size:expr) => {{
        static mut CHANNEL: $crate::channel::Channel<$type, $size> =
            $crate::channel::Channel::new();

        static CHECK: $crate::portable_atomic::AtomicU8 = $crate::portable_atomic::AtomicU8::new(0);

        $crate::channel::critical_section::with(|_| {
            if CHECK.load(::core::sync::atomic::Ordering::Relaxed) != 0 {
                panic!("call to the same `make_channel` instance twice");
            }

            CHECK.store(1, ::core::sync::atomic::Ordering::Relaxed);
        });

        // SAFETY: This is safe as we hide the static mut from others to access it.
        // Only this point is where the mutable access happens.
        #[allow(static_mut_refs)]
        unsafe {
            CHANNEL.split()
        }
    }};
}

// -------- Sender

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
pub struct Sender<'a, T, const N: usize>(&'a Channel<T, N>);

unsafe impl<T, const N: usize> Send for Sender<'_, T, N> {}

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

/// This is needed to make the async closure in `send` accept that we "share"
/// the link possible between threads.
#[derive(Clone)]
struct SlotPtr(*mut Option<u8>);

impl SlotPtr {
    /// Replace the value of this slot with `new_value`, and return
    /// the old value.
    ///
    /// SAFETY: the pointer in this `SlotPtr` must be valid for writes.
    unsafe fn replace(
        &mut self,
        new_value: Option<u8>,
        _cs: critical_section::CriticalSection,
    ) -> Option<u8> {
        // SAFETY: the critical section guarantees exclusive access, and the
        // caller guarantees that the pointer is valid.
        self.replace_exclusive(new_value)
    }

    /// Replace the value of this slot with `new_value`, and return
    /// the old value.
    ///
    /// SAFETY: the pointer in this `SlotPtr` must be valid for writes, and the caller must guarantee exclusive
    /// access to the underlying value..
    unsafe fn replace_exclusive(&mut self, new_value: Option<u8>) -> Option<u8> {
        // SAFETY: the caller has ensured that we have exclusive access & that
        // the pointer is valid.
        unsafe { core::ptr::replace(self.0, new_value) }
    }
}

unsafe impl Send for SlotPtr {}

unsafe impl Sync for SlotPtr {}

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
    #[inline(always)]
    fn send_footer(&mut self, idx: u8, val: T) {
        // Write the value to the slots, note; this memcpy is not under a critical section.
        unsafe {
            let first_element = self.0.slots.get_unchecked(idx as usize).get_mut();
            let ptr = first_element.deref().as_mut_ptr();
            ptr::write(ptr, val)
        }

        // Write the value into the ready queue.
        critical_section::with(|cs| {
            // SAFETY: `self.0.readyq` is not called recursively.
            unsafe {
                self.0.readyq(cs, |readyq| {
                    assert!(!readyq.is_full());
                    // SAFETY: ready is not full.
                    readyq.push_back_unchecked(idx);
                });
            }
        });

        fence(Ordering::SeqCst);

        // If there is a receiver waker, wake it.
        self.0.receiver_waker.wake();
    }

    /// Try to send a value, non-blocking. If the channel is full this will return an error.
    pub fn try_send(&mut self, val: T) -> Result<(), TrySendError<T>> {
        // If the wait queue is not empty, we can't try to push into the queue.
        if !self.0.wait_queue.is_empty() {
            return Err(TrySendError::Full(val));
        }

        // No receiver available.
        if self.is_closed() {
            return Err(TrySendError::NoReceiver(val));
        }

        let free_slot = critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.freeq` is not called recursively.
            self.0.freeq(cs, |q| q.pop_front())
        });

        let idx = if let Some(idx) = free_slot {
            idx
        } else {
            return Err(TrySendError::Full(val));
        };

        self.send_footer(idx, val);

        Ok(())
    }

    /// Send a value. If there is no place left in the queue this will wait until there is.
    /// If the receiver does not exist this will return an error.
    pub async fn send(&mut self, val: T) -> Result<(), NoReceiver<T>> {
        let mut free_slot_ptr: Option<u8> = None;
        let mut link_ptr: Option<Link<WaitQueueData>> = None;

        // Make this future `Drop`-safe.
        // SAFETY(link_ptr): Shadow the original definition of `link_ptr` so we can't abuse it.
        let mut link_ptr = LinkPtr(core::ptr::addr_of_mut!(link_ptr));
        // SAFETY(freed_slot): Shadow the original definition of `free_slot_ptr` so we can't abuse it.
        let mut free_slot_ptr = SlotPtr(core::ptr::addr_of_mut!(free_slot_ptr));

        let mut link_ptr2 = link_ptr.clone();
        let mut free_slot_ptr2 = free_slot_ptr.clone();
        let dropper = OnDrop::new(|| {
            // SAFETY: We only run this closure and dereference the pointer if we have
            // exited the `poll_fn` below in the `drop(dropper)` call. The other dereference
            // of this pointer is in the `poll_fn`.
            if let Some(link) = unsafe { link_ptr2.get() } {
                link.remove_from_list(&self.0.wait_queue);
            }

            // Return our potentially-unused free slot.
            // Since we are certain that our link has been removed from the list (either
            // pop-ed or removed just above), we have exclusive access to the free slot pointer.
            if let Some(freed_slot) = unsafe { free_slot_ptr2.replace_exclusive(None) } {
                // SAFETY: freed slot is passed to us from `return_free_slot`, which either
                // directly (through `try_recv`), or indirectly (through another `return_free_slot`)
                // comes from `readyq`.
                unsafe { self.0.return_free_slot(freed_slot) };
            }
        });

        let idx = poll_fn(|cx| {
            //  Do all this in one critical section, else there can be race conditions
            critical_section::with(|cs| {
                if self.is_closed() {
                    return Poll::Ready(Err(()));
                }

                let wq_empty = self.0.wait_queue.is_empty();
                // SAFETY: `self.0.freeq` is not called recursively.
                let freeq_empty = unsafe { self.0.freeq(cs, |q| q.is_empty()) };

                // SAFETY: This pointer is only dereferenced here and on drop of the future
                // which happens outside this `poll_fn`'s stack frame.
                let link = unsafe { link_ptr.get() };

                // We are already in the wait queue.
                if let Some(queue_link) = link {
                    if queue_link.is_popped() {
                        // SAFETY: `free_slot_ptr` is valid for writes until the end of this future.
                        let slot = unsafe { free_slot_ptr.replace(None, cs) };

                        // Our link was popped, so it is most definitely not in the list.
                        // We can safely & correctly `take` it to prevent ourselves from
                        // redundantly attempting to remove it from the list a 2nd time.
                        link.take();

                        // If our link is popped, then:
                        // 1. We were popped by `return_free_lot` and provided us with a slot.
                        // 2. We were popped by `Receiver::drop` and it did not provide us with a slot, and the channel is closed.
                        if let Some(slot) = slot {
                            Poll::Ready(Ok(slot))
                        } else {
                            Poll::Ready(Err(()))
                        }
                    } else {
                        Poll::Pending
                    }
                }
                // We are not in the wait queue, but others are, or there is currently no free
                // slot available.
                else if !wq_empty || freeq_empty {
                    // Place the link in the wait queue.
                    let link_ref =
                        link.insert(Link::new((cx.waker().clone(), free_slot_ptr.clone())));

                    // SAFETY(new_unchecked): The address to the link is stable as it is defined
                    // outside this stack frame.
                    // SAFETY(push): `link_ref` lifetime comes from `link_ptr` and `free_slot_ptr` that
                    // are shadowed and we make sure in `dropper` that the link is removed from the queue
                    // before dropping `link_ptr` AND `dropper` makes sure that the shadowed
                    // `ptr`s live until the end of the stack frame.
                    unsafe { self.0.wait_queue.push(Pin::new_unchecked(link_ref)) };

                    Poll::Pending
                }
                // We are not in the wait queue, no one else is waiting, and there is a free slot available.
                else {
                    // SAFETY: `self.0.freeq` is not called recursively.
                    unsafe {
                        self.0.freeq(cs, |freeq| {
                            assert!(!freeq.is_empty());
                            // SAFETY: `freeq` is non-empty
                            let slot = freeq.pop_back_unchecked();
                            Poll::Ready(Ok(slot))
                        })
                    }
                }
            })
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
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.receiver_dropped` is not called recursively.
            self.0.receiver_dropped(cs, |v| *v)
        })
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.freeq` is not called recursively.
            self.0.freeq(cs, |v| v.is_empty())
        })
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.freeq` is not called recursively.
            self.0.freeq(cs, |v| v.is_full())
        })
    }
}

impl<T, const N: usize> Drop for Sender<'_, T, N> {
    fn drop(&mut self) {
        // Count down the reference counter
        let num_senders = critical_section::with(|cs| {
            unsafe {
                // SAFETY: `self.0.num_senders` is not called recursively.
                self.0.num_senders(cs, |s| {
                    *s -= 1;
                    *s
                })
            }
        });

        // If there are no senders, wake the receiver to do error handling.
        if num_senders == 0 {
            self.0.receiver_waker.wake();
        }
    }
}

impl<T, const N: usize> Clone for Sender<'_, T, N> {
    fn clone(&self) -> Self {
        // Count up the reference counter
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.num_senders` is not called recursively.
            self.0.num_senders(cs, |v| *v += 1);
        });

        Self(self.0)
    }
}

// -------- Receiver

/// A receiver of the channel. There can only be one receiver at any time.
pub struct Receiver<'a, T, const N: usize>(&'a Channel<T, N>);

unsafe impl<T, const N: usize> Send for Receiver<'_, T, N> {}

impl<T, const N: usize> core::fmt::Debug for Receiver<'_, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Receiver")
    }
}

#[cfg(feature = "defmt-03")]
impl<T, const N: usize> defmt::Format for Receiver<'_, T, N> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Receiver",)
    }
}

/// Possible receive errors.
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReceiveError {
    /// Error state for when all senders has been dropped.
    NoSender,
    /// Error state for when the queue is empty.
    Empty,
}

impl<T, const N: usize> Receiver<'_, T, N> {
    /// Receives a value if there is one in the channel, non-blocking.
    pub fn try_recv(&mut self) -> Result<T, ReceiveError> {
        // Try to get a ready slot.
        let ready_slot = critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.readyq` is not called recursively.
            self.0.readyq(cs, |q| q.pop_front())
        });

        if let Some(rs) = ready_slot {
            // Read the value from the slots, note; this memcpy is not under a critical section.
            let r = unsafe {
                let first_element = self.0.slots.get_unchecked(rs as usize).get_mut();
                let ptr = first_element.deref().as_ptr();
                ptr::read(ptr)
            };

            // Return the index to the free queue after we've read the value.
            // SAFETY: `rs` comes directly from `readyq`.
            unsafe { self.0.return_free_slot(rs) };

            Ok(r)
        } else if self.is_closed() {
            Err(ReceiveError::NoSender)
        } else {
            Err(ReceiveError::Empty)
        }
    }

    /// Receives a value, waiting if the queue is empty.
    /// If all senders are dropped this will error with `NoSender`.
    pub async fn recv(&mut self) -> Result<T, ReceiveError> {
        // There was nothing in the queue, setup the waiting.
        poll_fn(|cx| {
            // Register waker.
            // TODO: Should it happen here or after the if? This might cause a spurious wake.
            self.0.receiver_waker.register(cx.waker());

            // Try to dequeue.
            match self.try_recv() {
                Ok(val) => {
                    return Poll::Ready(Ok(val));
                }
                Err(ReceiveError::NoSender) => {
                    return Poll::Ready(Err(ReceiveError::NoSender));
                }
                _ => {}
            }

            Poll::Pending
        })
        .await
    }

    /// Returns true if there are no `Sender`s.
    pub fn is_closed(&self) -> bool {
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.num_senders` is not called recursively.
            self.0.num_senders(cs, |v| *v == 0)
        })
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.readyq` is not called recursively.
            self.0.readyq(cs, |v| v.is_full())
        })
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.readyq` is not called recursively.
            self.0.readyq(cs, |v| v.is_empty())
        })
    }
}

impl<T, const N: usize> Drop for Receiver<'_, T, N> {
    fn drop(&mut self) {
        // Mark the receiver as dropped and wake all waiters
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.0.receiver_dropped` is not called recursively.
            self.0.receiver_dropped(cs, |v| *v = true);
        });

        while let Some((waker, _)) = self.0.wait_queue.pop() {
            waker.wake();
        }
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use core::sync::atomic::AtomicBool;
    use std::sync::Arc;

    use cassette::Cassette;

    use super::*;

    #[test]
    fn empty() {
        let (mut s, mut r) = make_channel!(u32, 10);

        assert!(s.is_empty());
        assert!(r.is_empty());

        s.try_send(1).unwrap();

        assert!(!s.is_empty());
        assert!(!r.is_empty());

        r.try_recv().unwrap();

        assert!(s.is_empty());
        assert!(r.is_empty());
    }

    #[test]
    fn full() {
        let (mut s, mut r) = make_channel!(u32, 3);

        for _ in 0..3 {
            assert!(!s.is_full());
            assert!(!r.is_full());

            s.try_send(1).unwrap();
        }

        assert!(s.is_full());
        assert!(r.is_full());

        for _ in 0..3 {
            r.try_recv().unwrap();

            assert!(!s.is_full());
            assert!(!r.is_full());
        }
    }

    #[test]
    fn send_recieve() {
        let (mut s, mut r) = make_channel!(u32, 10);

        for i in 0..10 {
            s.try_send(i).unwrap();
        }

        assert_eq!(s.try_send(11), Err(TrySendError::Full(11)));

        for i in 0..10 {
            assert_eq!(r.try_recv().unwrap(), i);
        }

        assert_eq!(r.try_recv(), Err(ReceiveError::Empty));
    }

    #[test]
    fn closed_recv() {
        let (s, mut r) = make_channel!(u32, 10);

        drop(s);

        assert!(r.is_closed());

        assert_eq!(r.try_recv(), Err(ReceiveError::NoSender));
    }

    #[test]
    fn closed_sender() {
        let (mut s, r) = make_channel!(u32, 10);

        drop(r);

        assert!(s.is_closed());

        assert_eq!(s.try_send(11), Err(TrySendError::NoReceiver(11)));
    }

    fn make() {
        let _ = make_channel!(u32, 10);
    }

    #[test]
    #[should_panic]
    fn double_make_channel() {
        make();
        make();
    }

    #[test]
    fn tuple_channel() {
        let _ = make_channel!((i32, u32), 10);
    }

    fn freeq<const N: usize, T, F, R>(channel: &Channel<T, N>, f: F) -> R
    where
        F: FnOnce(&mut Deque<u8, N>) -> R,
    {
        critical_section::with(|cs| unsafe { channel.freeq(cs, f) })
    }

    #[test]
    fn dropping_waked_send_returns_freeq_item() {
        let (mut tx, mut rx) = make_channel!(u8, 1);

        tx.try_send(0).unwrap();
        assert!(freeq(&rx.0, |q| q.is_empty()));

        // Running this in a separate thread scope to ensure that `pinned_future` is dropped fully.
        //
        // Calling drop explicitly gets hairy because dropping things behind a `Pin` is not easy.
        std::thread::scope(|scope| {
            scope.spawn(|| {
                let pinned_future = core::pin::pin!(tx.send(1));
                let mut future = Cassette::new(pinned_future);

                future.poll_on();

                assert!(freeq(&rx.0, |q| q.is_empty()));
                assert!(!rx.0.wait_queue.is_empty());

                assert_eq!(rx.try_recv(), Ok(0));

                assert!(freeq(&rx.0, |q| q.is_empty()));
            });
        });

        assert!(!freeq(&rx.0, |q| q.is_empty()));

        // Make sure that rx & tx are alive until here for good measure.
        drop((tx, rx));
    }

    #[derive(Debug)]
    struct SetToTrueOnDrop(Arc<AtomicBool>);

    impl Drop for SetToTrueOnDrop {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn non_popped_item_is_dropped() {
        let mut channel: Channel<SetToTrueOnDrop, 1> = Channel::new();

        let (mut tx, rx) = channel.split();

        let value = Arc::new(AtomicBool::new(false));
        tx.try_send(SetToTrueOnDrop(value.clone())).unwrap();

        drop((tx, rx));
        drop(channel);

        assert!(value.load(Ordering::SeqCst));
    }

    #[test]
    pub fn cleared_item_is_dropped() {
        let mut channel: Channel<SetToTrueOnDrop, 1> = Channel::new();

        let (mut tx, rx) = channel.split();

        let value = Arc::new(AtomicBool::new(false));
        tx.try_send(SetToTrueOnDrop(value.clone())).unwrap();

        drop((tx, rx));

        assert!(!value.load(Ordering::SeqCst));

        channel.clear();

        assert!(value.load(Ordering::SeqCst));
    }

    #[test]
    #[should_panic]
    pub fn splitting_non_empty_channel_panics() {
        let mut channel: Channel<(), 1> = Channel::new();

        let (mut tx, rx) = channel.split();

        tx.try_send(()).unwrap();

        drop((tx, rx));

        channel.split();
    }

    #[test]
    pub fn splitting_empty_channel_works() {
        let mut channel: Channel<(), 1> = Channel::new();

        let (mut tx, rx) = channel.split();

        tx.try_send(()).unwrap();

        drop((tx, rx));

        channel.clear();
        channel.split();
    }
}

#[cfg(not(loom))]
#[cfg(test)]
mod tokio_tests {
    #[tokio::test]
    async fn stress_channel() {
        const NUM_RUNS: usize = 1_000;
        const QUEUE_SIZE: usize = 10;

        let (s, mut r) = make_channel!(u32, QUEUE_SIZE);
        let mut v = std::vec::Vec::new();

        for i in 0..NUM_RUNS {
            let mut s = s.clone();

            v.push(tokio::spawn(async move {
                s.send(i as _).await.unwrap();
            }));
        }

        let mut map = std::collections::BTreeSet::new();

        for _ in 0..NUM_RUNS {
            map.insert(r.recv().await.unwrap());
        }

        assert_eq!(map.len(), NUM_RUNS);

        for v in v {
            v.await.unwrap();
        }
    }
}

#[cfg(test)]
#[cfg(loom)]
mod loom_test {
    use cassette::Cassette;
    use loom::thread;

    #[macro_export]
    #[allow(missing_docs)]
    macro_rules! make_loom_channel {
        ($type:ty, $size:expr) => {{
            let channel: crate::channel::Channel<$type, $size> = super::Channel::new();
            let boxed = Box::new(channel);
            let boxed = Box::leak(boxed);

            // SAFETY: This is safe as we hide the static mut from others to access it.
            // Only this point is where the mutable access happens.
            boxed.split()
        }};
    }

    // This test tests the following scenarios:
    // 1. Receiver is dropped while concurrent senders are waiting to send.
    // 2. Concurrent senders are competing for the same free slot.
    #[test]
    pub fn concurrent_send_while_full_and_drop() {
        loom::model(|| {
            let (mut tx, mut rx) = make_loom_channel!([u8; 20], 1);
            let mut cloned = tx.clone();

            tx.try_send([1; 20]).unwrap();

            let handle1 = thread::spawn(move || {
                let future = std::pin::pin!(tx.send([1; 20]));
                let mut future = Cassette::new(future);
                if future.poll_on().is_none() {
                    future.poll_on();
                }
            });

            rx.try_recv().ok();

            let future = std::pin::pin!(cloned.send([1; 20]));
            let mut future = Cassette::new(future);
            if future.poll_on().is_none() {
                future.poll_on();
            }

            drop(rx);

            handle1.join().unwrap();
        });
    }
}
