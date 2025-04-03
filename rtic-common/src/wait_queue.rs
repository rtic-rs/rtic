//! A wait queue implementation using a doubly linked list.

use core::{
    future::poll_fn,
    marker::PhantomPinned,
    pin::{pin, Pin},
    ptr::null_mut,
    task::{Poll, Waker},
};
use critical_section as cs;
use portable_atomic::{AtomicBool, AtomicPtr, Ordering};

use crate::dropper::OnDropWith;

/// A helper definition of a wait queue.
pub type WaitQueue = DoublyLinkedList<Waker>;

/// An atomic, doubly linked, FIFO list for a wait queue.
///
/// Atomicity is guaranteed by short [`critical_section`]s, so this list is _not_ lock free,
/// but it will not deadlock.
pub struct DoublyLinkedList<T> {
    head: AtomicPtr<Link<T>>, // UnsafeCell<*mut Link<T>>
    tail: AtomicPtr<Link<T>>,
}

impl<T> DoublyLinkedList<T> {
    /// Create a new linked list.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(null_mut()),
            tail: AtomicPtr::new(null_mut()),
        }
    }
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> DoublyLinkedList<T> {
    const R: Ordering = Ordering::Relaxed;

    /// Pop the first element in the queue.
    pub fn pop(&self) -> Option<T> {
        cs::with(|_| {
            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            let head = self.head.load(Self::R);

            // SAFETY: `as_ref` is safe as `insert` requires a valid reference to a link
            if let Some(head_ref) = unsafe { head.as_ref() } {
                // Move head to the next element
                self.head.store(head_ref.next.load(Self::R), Self::R);

                // We read the value at head
                let head_val = head_ref.val.clone();

                let tail = self.tail.load(Self::R);
                if head == tail {
                    // The queue is empty
                    self.tail.store(null_mut(), Self::R);
                }

                if let Some(next_ref) = unsafe { head_ref.next.load(Self::R).as_ref() } {
                    next_ref.prev.store(null_mut(), Self::R);
                }

                // Clear the pointers in the node.
                head_ref.next.store(null_mut(), Self::R);
                head_ref.prev.store(null_mut(), Self::R);
                head_ref.is_popped.store(true, Self::R);

                return Some(head_val);
            }

            None
        })
    }

    /// Put an element at the back of the queue.
    ///
    /// # Safety
    ///
    /// The link must live until it is removed from the queue.
    pub unsafe fn push(&self, link: Pin<&Link<T>>) {
        cs::with(|_| {
            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            let tail = self.tail.load(Self::R);

            // SAFETY: This datastructure does not move the underlying value.
            let link = link.get_ref();

            if let Some(tail_ref) = unsafe { tail.as_ref() } {
                // Queue is not empty
                link.prev.store(tail, Self::R);
                self.tail.store(link as *const _ as *mut _, Self::R);
                tail_ref.next.store(link as *const _ as *mut _, Self::R);
            } else {
                // Queue is empty
                self.tail.store(link as *const _ as *mut _, Self::R);
                self.head.store(link as *const _ as *mut _, Self::R);
            }
        });
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.head.load(Self::R).is_null()
    }
}

/// A link in the linked list.
pub struct Link<T> {
    pub(crate) val: T,
    next: AtomicPtr<Link<T>>,
    prev: AtomicPtr<Link<T>>,
    is_popped: AtomicBool,
    _up: PhantomPinned,
}

impl<T: Clone> Link<T> {
    const R: Ordering = Ordering::Relaxed;

    /// Create a new link.
    pub const fn new(val: T) -> Self {
        Self {
            val,
            next: AtomicPtr::new(null_mut()),
            prev: AtomicPtr::new(null_mut()),
            is_popped: AtomicBool::new(false),
            _up: PhantomPinned,
        }
    }

    /// Return true if this link has been poped from the list.
    pub fn is_popped(&self) -> bool {
        self.is_popped.load(Self::R)
    }

    /// Remove this link from a linked list.
    pub fn remove_from_list(&self, list: &DoublyLinkedList<T>) {
        cs::with(|_| {
            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            let prev = self.prev.load(Self::R);
            let next = self.next.load(Self::R);
            self.is_popped.store(true, Self::R);

            match unsafe { (prev.as_ref(), next.as_ref()) } {
                (None, None) => {
                    // Not in the list or alone in the list, check if list head == node address
                    let sp = self as *const _;

                    if sp == list.head.load(Ordering::Relaxed) {
                        list.head.store(null_mut(), Self::R);
                        list.tail.store(null_mut(), Self::R);
                    }
                }
                (None, Some(next_ref)) => {
                    // First in the list
                    next_ref.prev.store(null_mut(), Self::R);
                    list.head.store(next, Self::R);
                }
                (Some(prev_ref), None) => {
                    // Last in the list
                    prev_ref.next.store(null_mut(), Self::R);
                    list.tail.store(prev, Self::R);
                }
                (Some(prev_ref), Some(next_ref)) => {
                    // Somewhere in the list

                    // Connect the `prev.next` and `next.prev` with each other to remove the node
                    prev_ref.next.store(next, Self::R);
                    next_ref.prev.store(prev, Self::R);
                }
            }
        })
    }
}

#[cfg(test)]
impl<T: core::fmt::Debug + Clone> DoublyLinkedList<T> {
    fn print(&self) {
        cs::with(|_| {
            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            let mut head = self.head.load(Self::R);
            let tail = self.tail.load(Self::R);

            println!(
                "List - h = 0x{:x}, t = 0x{:x}",
                head as usize, tail as usize
            );

            let mut i = 0;

            // SAFETY: `as_ref` is safe as `insert` requires a valid reference to a link
            while let Some(head_ref) = unsafe { head.as_ref() } {
                println!(
                    "    {}: {:?}, s = 0x{:x}, n = 0x{:x}, p = 0x{:x}",
                    i,
                    head_ref.val,
                    head as usize,
                    head_ref.next.load(Ordering::Relaxed) as usize,
                    head_ref.prev.load(Ordering::Relaxed) as usize
                );

                head = head_ref.next.load(Self::R);

                i += 1;
            }
        });
    }
}

impl DoublyLinkedList<Waker> {
    /// Wait until `f` returns `Some`.
    pub async fn wait_until<T, F: FnMut() -> Option<T>>(&self, mut f: F) -> T {
        let link_place = pin!(None::<Link<Waker>>);

        let mut link_guard = OnDropWith::new(link_place, |link| {
            if let Some(link) = link.as_ref().as_pin_ref() {
                link.remove_from_list(self);
            }
            link.set(None);
        });

        poll_fn(move |cx| {
            // clean up the old link, because we are going to invalidate it.
            // we are doing it before returning `Poll::Ready` to handle cases
            // where the future is polled after it is completed.
            link_guard.execute();

            if let Some(val) = f() {
                return Poll::Ready(val);
            }

            // note: we may introduce a more complex logic to try to reuse the old link
            // with the old waker by using `Waker::will_wake` to avoid `Waker::clone`,
            // but it is probably not needed as Rtic's `waker` is cheap to clone.

            // By the contract, each poll we should update the waker.
            let new_link = Link::new(cx.waker().clone());

            // Store the link into the pinned place.
            link_guard.set(Some(new_link));

            let new_link_pinned = link_guard.as_ref().as_pin_ref().expect("We just set it");

            // SAFETY: we guarantee that `link` will live until removed by cleaning it up
            // in the destructor of the future and that destructor is guaranteed to run
            // before it's memory is reused or invalidated because the future is pinned.
            unsafe { self.push(new_link_pinned) };

            Poll::Pending
        })
        .await
    }
}

#[cfg(test)]
impl<T: core::fmt::Debug + Clone> Link<T> {
    fn print(&self) {
        cs::with(|_| {
            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            println!("Link:");

            println!(
                "    val = {:?}, n = 0x{:x}, p = 0x{:x}",
                self.val,
                self.next.load(Ordering::Relaxed) as usize,
                self.prev.load(Ordering::Relaxed) as usize
            );
        });
    }
}

/// Test that the future returned by `wait_until` is not `Unpin`.
/// ```compile_fail
/// fn test_unpin(list: &rtic_common::wait_queue::DoublyLinkedList<core::task::Waker>, cx: &mut core::task::Context) {
///     let mut wait_until_future = list.wait_until(|| None::<()>);
///     let pinned = core::pin::Pin::new(&mut wait_until_future);
///     core::future::Future::poll(pinned, cx);
///  }
/// ```
/// This test will ensure that previous test failed because of `pin`.
/// ```
/// fn test_unpin(list: &rtic_common::wait_queue::DoublyLinkedList<core::task::Waker>, cx: &mut core::task::Context) {
///     let mut wait_until_future = list.wait_until(|| None::<()>);
///     let pinned = core::pin::pin!(wait_until_future);
///     core::future::Future::poll(pinned, cx);
///  }
/// ```
mod compile_fail_test {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_list() {
        let wq = DoublyLinkedList::<u32>::new();

        let i1 = Link::new(10);
        let i2 = Link::new(11);
        let i3 = Link::new(12);
        let i4 = Link::new(13);
        let i5 = Link::new(14);

        unsafe { wq.push(Pin::new_unchecked(&i1)) };
        unsafe { wq.push(Pin::new_unchecked(&i2)) };
        unsafe { wq.push(Pin::new_unchecked(&i3)) };
        unsafe { wq.push(Pin::new_unchecked(&i4)) };
        unsafe { wq.push(Pin::new_unchecked(&i5)) };

        wq.print();

        wq.pop();
        i1.print();

        wq.print();

        i4.remove_from_list(&wq);

        wq.print();

        // i1.remove_from_list(&wq);
        // wq.print();

        println!("i2");
        i2.remove_from_list(&wq);
        wq.print();

        println!("i3");
        i3.remove_from_list(&wq);
        wq.print();

        println!("i5");
        i5.remove_from_list(&wq);
        wq.print();
    }
}
