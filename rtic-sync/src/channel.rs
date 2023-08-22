//! An async aware MPSC channel that can be used on no-alloc systems.

use core::{
    cell::UnsafeCell,
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
use rtic_common::waker_registration::CriticalSectionWakerRegistration as WakerRegistration;
use rtic_common::{
    dropper::OnDrop,
    wait_queue::{Link, WaitQueue},
};

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
    pub fn split(&mut self) -> (Sender<'_, T, N>, Receiver<'_, T, N>) {
        // Fill free queue
        for idx in 0..N as u8 {
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
        static mut CHANNEL: $crate::channel::Channel<$type, $size> =
            $crate::channel::Channel::new();

        static CHECK: ::core::sync::atomic::AtomicU8 = ::core::sync::atomic::AtomicU8::new(0);

        $crate::channel::critical_section::with(|_| {
            if CHECK.load(::core::sync::atomic::Ordering::Relaxed) != 0 {
                panic!("call to the same `make_channel` instance twice");
            }

            CHECK.store(1, ::core::sync::atomic::Ordering::Relaxed);
        });

        // SAFETY: This is safe as we hide the static mut from others to access it.
        // Only this point is where the mutable access happens.
        unsafe { CHANNEL.split() }
    }};
}

// -------- Sender

/// Error state for when the receiver has been dropped.
pub struct NoReceiver<T>(pub T);

/// Errors that 'try_send` can have.
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

unsafe impl<'a, T, const N: usize> Send for Sender<'a, T, N> {}

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

impl<'a, T, const N: usize> core::fmt::Debug for Sender<'a, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Sender")
    }
}

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
        critical_section::with(|cs| {
            debug_assert!(!self.0.access(cs).readyq.is_full());
            unsafe { self.0.access(cs).readyq.push_back_unchecked(idx) }
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
            if let Some(idx) = critical_section::with(|cs| self.0.access(cs).freeq.pop_front()) {
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
                let fq_empty = self.0.access(cs).freeq.is_empty();
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

                debug_assert!(!self.0.access(cs).freeq.is_empty());
                // Get index as the queue is guaranteed not empty and the wait queue is empty
                let idx = unsafe { self.0.access(cs).freeq.pop_front_unchecked() };

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

unsafe impl<'a, T, const N: usize> Send for Receiver<'a, T, N> {}

impl<'a, T, const N: usize> core::fmt::Debug for Receiver<'a, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Receiver")
    }
}

/// Possible receive errors.
#[derive(Debug, PartialEq, Eq)]
pub enum ReceiveError {
    /// Error state for when all senders has been dropped.
    NoSender,
    /// Error state for when the queue is empty.
    Empty,
}

impl<'a, T, const N: usize> Receiver<'a, T, N> {
    /// Receives a value if there is one in the channel, non-blocking.
    pub fn try_recv(&mut self) -> Result<T, ReceiveError> {
        // Try to get a ready slot.
        let ready_slot = critical_section::with(|cs| self.0.access(cs).readyq.pop_front());

        if let Some(rs) = ready_slot {
            // Read the value from the slots, note; this memcpy is not under a critical section.
            let r = unsafe { ptr::read(self.0.slots.get_unchecked(rs as usize).get() as *const T) };

            // Return the index to the free queue after we've read the value.
            critical_section::with(|cs| {
                debug_assert!(!self.0.access(cs).freeq.is_full());
                unsafe { self.0.access(cs).freeq.push_back_unchecked(rs) }
            });

            fence(Ordering::SeqCst);

            // If someone is waiting in the WaiterQueue, wake the first one up.
            if let Some(wait_head) = self.0.wait_queue.pop() {
                wait_head.wake();
            }

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
        critical_section::with(|cs| *self.0.access(cs).num_senders == 0)
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        critical_section::with(|cs| self.0.access(cs).readyq.is_full())
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

#[cfg(test)]
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

    fn make() {
        let _ = make_channel!(u32, 10);
    }

    #[test]
    #[should_panic]
    fn double_make_channel() {
        make();
        make();
    }
}
