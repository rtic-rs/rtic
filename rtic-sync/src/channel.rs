//! An async aware MPSC channel that can be used on no-alloc systems.

use core::{
    future::poll_fn,
    mem::MaybeUninit,
    pin::Pin,
    ptr,
    sync::atomic::{fence, Ordering},
    task::{Poll, Waker},
};

#[cfg(not(loom))]
use rtic_common::unsafecell::UnsafeCell;

#[cfg(loom)]
use loom::cell::UnsafeCell;

#[doc(hidden)]
pub use critical_section;

use heapless::Deque;

use rtic_common::waker_registration::CriticalSectionWakerRegistration as WakerRegistration;

use rtic_common::{
    dropper::OnDrop,
    wait_queue::{Link, WaitQueue},
};

#[cfg(feature = "defmt-03")]
use crate::defmt;

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
        // Fill free queue
        for idx in 0..N as u8 {
            self.freeq.with_mut(|freeq| {
                let freeq = unsafe { &mut *freeq };
                assert!(!freeq.is_full());

                // SAFETY: This safe as the loop goes from 0 to the capacity of the underlying queue.
                unsafe {
                    freeq.push_back_unchecked(idx);
                }
            });
        }

        self.freeq.with(|freeq| {
            assert!(unsafe { &*freeq }.is_full());
        });

        // There is now 1 sender
        self.num_senders.with_mut(|v| unsafe {
            *v = 1;
        });

        (Sender(self), Receiver(self))
    }

    fn freeq<F, R>(&self, _cs: critical_section::CriticalSection, f: F) -> R
    where
        F: FnOnce(&mut Deque<u8, N>) -> R,
    {
        self.freeq.with_mut(|freeq| {
            let queue = unsafe { &mut *freeq };
            f(queue)
        })
    }

    fn readyq<F, R>(&self, _cs: critical_section::CriticalSection, f: F) -> R
    where
        F: FnOnce(&mut Deque<u8, N>) -> R,
    {
        self.readyq.with_mut(|readyq| {
            let queue = unsafe { &mut *readyq };
            f(queue)
        })
    }

    fn receiver_dropped<F, R>(&self, _cs: critical_section::CriticalSection, f: F) -> R
    where
        F: FnOnce(&mut bool) -> R,
    {
        self.receiver_dropped.with_mut(|receiver_dropped| {
            let receiver_dropped = unsafe { &mut *receiver_dropped };
            f(receiver_dropped)
        })
    }

    fn num_senders<F, R>(&self, _cs: critical_section::CriticalSection, f: F) -> R
    where
        F: FnOnce(&mut usize) -> R,
    {
        self.num_senders.with_mut(|num_senders| {
            let num_senders = unsafe { &mut *num_senders };
            f(num_senders)
        })
    }
}

/// Creates a split channel with `'static` lifetime.R
#[macro_export]
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
struct LinkPtr(*mut Option<Link<Waker>>);

impl LinkPtr {
    /// This will dereference the pointer stored within and give out an `&mut`.
    unsafe fn get(&mut self) -> &mut Option<Link<Waker>> {
        &mut *self.0
    }
}

unsafe impl Send for LinkPtr {}

unsafe impl Sync for LinkPtr {}

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
            let ptr = (&raw const self.0.slots[0]).add(idx as _);
            let ptr = ptr as *mut UnsafeCell<MaybeUninit<T>>;
            ptr::write(ptr, UnsafeCell::new(MaybeUninit::new(val)));
        }

        // Write the value into the ready queue.
        critical_section::with(|cs| {
            assert!(!self.0.readyq(cs, |q| q.is_full()));
            unsafe { self.0.readyq(cs, |q| q.push_back_unchecked(idx)) }
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

        let idx =
            if let Some(idx) = critical_section::with(|cs| self.0.freeq(cs, |q| q.pop_front())) {
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
                link.remove_from_list(&self.0.wait_queue);
            }
        });

        let idx = poll_fn(|cx| {
            if self.is_closed() {
                return Poll::Ready(Err(()));
            }

            //  Do all this in one critical section, else there can be race conditions
            let queue_idx = critical_section::with(|cs| {
                let wq_empty = self.0.wait_queue.is_empty();
                let fq_empty = self.0.freeq(cs, |q| q.is_empty());

                if !wq_empty || fq_empty {
                    // SAFETY: This pointer is only dereferenced here and on drop of the future
                    // which happens outside this `poll_fn`'s stack frame.
                    let link = unsafe { link_ptr.get() };
                    if let Some(link) = link {
                        if !link.is_popped() {
                            return None;
                        } else {
                            // Fall through to dequeue
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
                        unsafe { self.0.wait_queue.push(Pin::new_unchecked(link_ref)) };

                        return None;
                    }
                }

                assert!(!self.0.freeq(cs, |q| q.is_empty()));
                // Get index as the queue is guaranteed not empty and the wait queue is empty
                let idx = unsafe { self.0.freeq(cs, |q| q.pop_front_unchecked()) };

                Some(idx)
            });

            if let Some(idx) = queue_idx {
                // Return the index
                Poll::Ready(Ok(idx))
            } else {
                Poll::Pending
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
        critical_section::with(|cs| self.0.receiver_dropped(cs, |v| *v))
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        critical_section::with(|cs| self.0.freeq(cs, |q| q.is_empty()))
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        critical_section::with(|cs| self.0.freeq(cs, |q| q.is_full()))
    }
}

impl<T, const N: usize> Drop for Sender<'_, T, N> {
    fn drop(&mut self) {
        // Count down the reference counter
        let num_senders = critical_section::with(|cs| {
            self.0.num_senders(cs, |v| {
                *v -= 1;
                *v
            })
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
        critical_section::with(|cs| self.0.num_senders(cs, |v| *v += 1));

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
        let ready_slot = critical_section::with(|cs| self.0.readyq(cs, |q| q.pop_front()));

        if let Some(rs) = ready_slot {
            // Read the value from the slots, note; this memcpy is not under a critical section.
            let r = unsafe {
                let ptr = (&raw const self.0.slots[0]).add(rs as _);
                ptr::read(ptr).into_inner().assume_init()
            };

            // Return the index to the free queue after we've read the value.
            critical_section::with(|cs| {
                self.0.freeq(cs, |freeq| {
                    assert!(!freeq.is_full());
                    unsafe { freeq.push_back_unchecked(rs) }
                });

                fence(Ordering::SeqCst);

                // If someone is waiting in the WaiterQueue, wake the first one up.
                if let Some(wait_head) = self.0.wait_queue.pop() {
                    wait_head.wake();
                }

                Ok(r)
            })
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
        critical_section::with(|cs| self.0.num_senders(cs, |v| *v == 0))
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        critical_section::with(|cs| self.0.readyq(cs, |q| q.is_full()))
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        critical_section::with(|cs| self.0.readyq(cs, |q| q.is_empty()))
    }
}

impl<T, const N: usize> Drop for Receiver<'_, T, N> {
    fn drop(&mut self) {
        // Mark the receiver as dropped and wake all waiters
        critical_section::with(|cs| self.0.receiver_dropped(cs, |v| *v = true));

        while let Some(waker) = self.0.wait_queue.pop() {
            waker.wake();
        }
    }
}

#[cfg(test)]
#[cfg(loom)]
mod loom_tests {
    #![allow(missing_docs)]

    use std::boxed::Box;

    use cassette::Cassette;

    #[macro_export]
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

    #[test]
    pub fn a_repro() {
        use loom::thread;

        loom::model(|| {
            let (mut spam_tx_send, mut spam_tx_recv) = make_loom_channel!([u8; 20], 1);

            spam_tx_send.try_send([1; 20]).unwrap();

            let handle = thread::spawn(move || {
                spam_tx_send.try_send([1; 20]).ok();

                let future = std::pin::pin!(spam_tx_send.send([1; 20]));
                let mut future = Cassette::new(future);
                if future.poll_on().is_none() {
                    future.poll_on();
                }
            });

            spam_tx_recv.try_recv().ok();
            spam_tx_recv.try_recv().ok();

            handle.join().unwrap();
        });
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
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
}

#[cfg(test)]
#[cfg(not(loom))]
mod stress_test {
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
