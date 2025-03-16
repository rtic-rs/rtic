use core::{
    mem::MaybeUninit,
    pin::Pin,
    ptr,
    sync::atomic::{fence, Ordering},
    task::Waker,
};

use heapless::Deque;
use rtic_common::{
    wait_queue::{DoublyLinkedList, Link},
    waker_registration::CriticalSectionWakerRegistration as WakerRegistration,
};

use super::{Receiver, Sender};

use crate::unsafecell::UnsafeCell;

pub(crate) type WaitQueueData = (Waker, FreeSlotPtr);
pub(crate) type WaitQueue = DoublyLinkedList<WaitQueueData>;

macro_rules! cs_access {
    ($name:ident, $field:ident, $type:ty) => {
        /// Access the value mutably.
        ///
        /// SAFETY: this function must not be called recursively within `f`.
        unsafe fn $name<F, R>(&self, _cs: critical_section::CriticalSection, f: F) -> R
        where
            F: FnOnce(&mut $type) -> R,
        {
            self.$field.with_mut(|v| {
                let v = unsafe { &mut *v };
                f(v)
            })
        }
    };
}

/// A free slot.
#[derive(Debug)]
pub(crate) struct FreeSlot(u8);

/// A pointer to a free slot.
///
/// This struct exists to enforce lifetime/safety requirements, and to ensure
/// that [`FreeSlot`]s can only be created/updated by this module.
#[derive(Clone)]
pub(crate) struct FreeSlotPtr(*mut Option<FreeSlot>);

impl FreeSlotPtr {
    /// SAFETY: `inner` must be valid until the [`Link`] containing this [`FreeSlotPtr`] is popped.
    /// Additionally, this [`FreeSlotPtr`] must have exclusive access to the data pointed to by
    /// `inner`.
    pub unsafe fn new(inner: *mut Option<FreeSlot>) -> Self {
        Self(inner)
    }

    /// Replace the value of this slot with `new_value`, and return
    /// the old value.
    ///
    /// SAFETY: the pointer in this [`FreeSlotPtr`] must be valid for writes.
    pub(crate) unsafe fn take(
        &mut self,
        cs: critical_section::CriticalSection,
    ) -> Option<FreeSlot> {
        self.replace(None, cs)
    }

    /// Replace the value of this slot with `new_value`, and return
    /// the old value.
    ///
    /// SAFETY: the pointer in this [`FreeSlotPtr`] must be valid for writes, and `new_value` must
    /// be obtained from `freeq`.
    unsafe fn replace(
        &mut self,
        new_value: Option<FreeSlot>,
        _cs: critical_section::CriticalSection,
    ) -> Option<FreeSlot> {
        // SAFETY: we are in a critical section.
        unsafe { core::ptr::replace(self.0, new_value) }
    }
}

unsafe impl Send for FreeSlotPtr {}

unsafe impl Sync for FreeSlotPtr {}

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

    /// Split the queue into a `Sender`/`Receiver` pair.
    pub fn split(&mut self) -> (Sender<'_, T, N>, Receiver<'_, T, N>) {
        // SAFETY: we have exclusive access to `self`.
        let freeq = self.freeq.get_mut();
        let freeq = unsafe { freeq.deref() };

        // Fill free queue
        for idx in 0..N as u8 {
            assert!(!freeq.is_full());

            // SAFETY: This safe as the loop goes from 0 to the capacity of the underlying queue.
            unsafe {
                freeq.push_back_unchecked(idx);
            }
        }

        assert!(freeq.is_full());

        // There is now 1 sender
        // SAFETY: we have exclusive access to `self`.
        unsafe { *self.num_senders.get_mut().deref() = 1 };

        (Sender(self), Receiver(self))
    }

    cs_access!(access_freeq, freeq, Deque<u8, N>);
    cs_access!(access_readyq, readyq, Deque<u8, N>);
    cs_access!(access_receiver_dropped, receiver_dropped, bool);
    cs_access!(access_num_senders, num_senders, usize);

    /// SAFETY: this function must not be called recursively in `f`.
    pub(crate) unsafe fn freeq<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Deque<u8, N>) -> R,
    {
        critical_section::with(|cs| self.access_freeq(cs, |v| f(&v)))
    }

    /// SAFETY: this function must not be called recursively in `f`.
    pub(crate) unsafe fn readyq<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Deque<u8, N>) -> R,
    {
        critical_section::with(|cs| self.access_readyq(cs, |v| f(&v)))
    }

    pub(crate) fn num_senders(&self) -> usize {
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.access_num_senders` is not called recursively.
            self.access_num_senders(cs, |v| *v)
        })
    }

    pub(crate) fn receiver_dropped(&self) -> bool {
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.receiver_dropped` is not called recursively.
            self.access_receiver_dropped(cs, |v| *v)
        })
    }

    /// Return free slot `slot` to the channel.
    ///
    /// This will do one of two things:
    /// 1. If there are any waiting `send`-ers, wake the longest-waiting one and hand it `slot`.
    /// 2. else, insert `slot` into the free queue.
    ///
    /// SAFETY: `slot` must be obtained from this exact channel instance.
    pub(crate) unsafe fn return_free_slot(&self, slot: FreeSlot) {
        critical_section::with(|cs| {
            fence(Ordering::SeqCst);

            // If a sender is waiting in the `wait_queue`, wake the first one up & hand it the free slot.
            if let Some((wait_head, mut freeq_slot)) = self.wait_queue.pop() {
                // SAFETY: `freeq_slot` is valid for writes: we are in a critical
                // section & the `FreeSlotPtr` lives for at least the duration of the wait queue link.
                unsafe { freeq_slot.replace(Some(slot), cs) };
                wait_head.wake();
            } else {
                // SAFETY: `self.freeq` is not called recursively.
                unsafe {
                    self.access_freeq(cs, |freeq| {
                        assert!(!freeq.is_full());
                        // SAFETY: `freeq` is not full.
                        freeq.push_back_unchecked(slot.0);
                    });
                }
            }
        });
    }

    /// Send a value using the given `slot` in this channel.
    ///
    /// SAFETY: `slot` must be obtained from this exact channel instance.
    #[inline(always)]
    pub(crate) unsafe fn send_value(&self, slot: FreeSlot, val: T) {
        let slot = slot.0;

        // Write the value to the slots, note; this memcpy is not under a critical section.
        unsafe {
            let first_element = self.slots.get_unchecked(slot as usize).get_mut();
            let ptr = first_element.deref().as_mut_ptr();
            ptr::write(ptr, val)
        }

        // Write the value into the ready queue.
        critical_section::with(|cs| {
            // SAFETY: `self.readyq` is not called recursively.
            unsafe {
                self.access_readyq(cs, |readyq| {
                    assert!(!readyq.is_full());
                    // SAFETY: ready is not full.
                    readyq.push_back_unchecked(slot);
                });
            }
        });

        fence(Ordering::SeqCst);

        // If there is a receiver waker, wake it.
        self.receiver_waker.wake();
    }

    /// Pop the value of a ready slot to make it available to a receiver.
    ///
    /// Internally, this function does these things:
    /// 1. Pop a ready slot from the ready queue.
    /// 2. If available, read the data from the backing slot storage.
    /// 3. If available, return the now-free slot to the free queue.
    pub(crate) fn receive_value(&self) -> Option<T> {
        let ready_slot = critical_section::with(|cs| unsafe {
            // SAFETY: `self.readyq` is not called recursively.
            self.access_readyq(cs, |q| q.pop_front())
        });

        if let Some(rs) = ready_slot {
            let r = unsafe {
                let first_element = self.slots.get_unchecked(rs as usize).get_mut();
                let ptr = first_element.deref().as_ptr();
                ptr::read(ptr)
            };

            // Return the index to the free queue after we've read the value.
            // SAFETY: `rs` is now a free slot obtained from this channel.
            unsafe { self.return_free_slot(FreeSlot(rs)) };

            Some(r)
        } else {
            None
        }
    }

    /// Register a new waiter in the wait queue.
    ///
    /// SAFETY: `link` must be valid until it is popped.
    pub(crate) unsafe fn push_wait_queue(&self, link: Pin<&Link<WaitQueueData>>) {
        self.wait_queue.push(link);
    }

    pub(crate) fn remove_from_wait_queue(&self, link: &Link<WaitQueueData>) {
        link.remove_from_list(&self.wait_queue);
    }

    /// Pop a free slot.
    pub(crate) fn pop_free_slot(&self) -> Option<FreeSlot> {
        let slot = critical_section::with(|cs| unsafe {
            // SAFETY: `self.freeq` is not called recursively.
            self.access_freeq(cs, |q| q.pop_front())
        });
        slot.map(FreeSlot)
    }

    pub(crate) fn drop_receiver(&self) {
        // Mark the receiver as dropped and wake all waiters
        critical_section::with(|cs| unsafe {
            // SAFTEY: `self.receiver_dropped` is not called recursively.
            self.access_receiver_dropped(cs, |v| *v = true)
        });

        while let Some((waker, _)) = self.wait_queue.pop() {
            waker.wake();
        }
    }

    pub(crate) fn register_receiver_waker(&self, waker: &Waker) {
        self.receiver_waker.register(waker);
    }

    pub(crate) fn drop_sender(&self) {
        // Count down the reference counter
        let num_senders = critical_section::with(|cs| unsafe {
            // SAFETY: `self.num_senders` is not called recursively.
            self.access_num_senders(cs, |s| {
                *s -= 1;
                *s
            })
        });

        // If there are no senders, wake the receiver to do error handling.
        if num_senders == 0 {
            self.receiver_waker.wake();
        }
    }

    pub(crate) fn clone_sender(&self) {
        // Count up the reference counter
        critical_section::with(|cs| unsafe {
            // SAFETY: `self.num_senders` is not called recursively.
            self.access_num_senders(cs, |v| *v += 1)
        });
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use crate::{
        channel::{ReceiveError, TrySendError},
        make_channel,
    };
    use cassette::Cassette;
    use heapless::Deque;

    use super::Channel;

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
        critical_section::with(|cs| unsafe { channel.access_freeq(cs, f) })
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
}

#[cfg(not(loom))]
#[cfg(test)]
mod tokio_tests {
    use crate::make_channel;

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
