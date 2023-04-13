use core::marker::PhantomPinned;
use core::pin::Pin;
use core::sync::atomic::{AtomicPtr, Ordering};
use critical_section as cs;

/// An atomic sorted linked list for the timer queue.
///
/// Atomicity is guaranteed using very short [`critical_section`]s, so this list is _not_
/// lock free, but it will not deadlock.
pub(crate) struct LinkedList<T> {
    head: AtomicPtr<Link<T>>,
}

impl<T> LinkedList<T> {
    /// Create a new linked list.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

impl<T: PartialOrd + Clone> LinkedList<T> {
    /// Pop the first element in the queue if the closure returns true.
    pub fn pop_if<F: FnOnce(&T) -> bool>(&self, f: F) -> Option<T> {
        cs::with(|_| {
            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            let head = self.head.load(Ordering::Relaxed);

            // SAFETY: `as_ref` is safe as `insert` requires a valid reference to a link
            if let Some(head) = unsafe { head.as_ref() } {
                if f(&head.val) {
                    // Move head to the next element
                    self.head
                        .store(head.next.load(Ordering::Relaxed), Ordering::Relaxed);

                    // We read the value at head
                    let head_val = head.val.clone();

                    return Some(head_val);
                }
            }
            None
        })
    }

    /// Delete a link at an address.
    pub fn delete(&self, addr: usize) {
        cs::with(|_| {
            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            let head = self.head.load(Ordering::Relaxed);

            // SAFETY: `as_ref` is safe as `insert` requires a valid reference to a link
            let head_ref = if let Some(head_ref) = unsafe { head.as_ref() } {
                head_ref
            } else {
                // 1. List is empty, do nothing
                return;
            };

            if head as *const _ as usize == addr {
                // 2. Replace head with head.next
                self.head
                    .store(head_ref.next.load(Ordering::Relaxed), Ordering::Relaxed);

                return;
            }

            // 3. search list for correct node
            let mut curr = head_ref;
            let mut next = head_ref.next.load(Ordering::Relaxed);

            // SAFETY: `as_ref` is safe as `insert` requires a valid reference to a link
            while let Some(next_link) = unsafe { next.as_ref() } {
                // Next is not null

                if next as *const _ as usize == addr {
                    curr.next
                        .store(next_link.next.load(Ordering::Relaxed), Ordering::Relaxed);

                    return;
                }

                // Continue searching
                curr = next_link;
                next = next_link.next.load(Ordering::Relaxed);
            }
        })
    }

    /// Insert a new link into the linked list.
    /// The return is (updated head, address), where the address of the link is for use
    /// with `delete`.
    ///
    /// SAFETY: The pinned link must live until it is removed from this list.
    pub unsafe fn insert(&self, val: Pin<&Link<T>>) -> (bool, usize) {
        cs::with(|_| {
            // SAFETY: This datastructure does not move the underlying value.
            let val = val.get_ref();
            let addr = val as *const _ as usize;

            // Make sure all previous writes are visible
            core::sync::atomic::fence(Ordering::SeqCst);

            let head = self.head.load(Ordering::Relaxed);

            // 3 cases to handle

            // 1. List is empty, write to head
            // SAFETY: `as_ref` is safe as `insert` requires a valid reference to a link
            let head_ref = if let Some(head_ref) = unsafe { head.as_ref() } {
                head_ref
            } else {
                self.head
                    .store(val as *const _ as *mut _, Ordering::Relaxed);
                return (true, addr);
            };

            // 2. val needs to go in first
            if val.val < head_ref.val {
                // Set current head as next of `val`
                val.next.store(head, Ordering::Relaxed);

                // `val` is now first in the queue
                self.head
                    .store(val as *const _ as *mut _, Ordering::Relaxed);

                return (true, addr);
            }

            // 3. search list for correct place
            let mut curr = head_ref;
            let mut next = head_ref.next.load(Ordering::Relaxed);

            // SAFETY: `as_ref` is safe as `insert` requires a valid reference to a link
            while let Some(next_link) = unsafe { next.as_ref() } {
                // Next is not null

                if val.val < next_link.val {
                    // Replace next with `val`
                    val.next.store(next, Ordering::Relaxed);

                    // Insert `val`
                    curr.next
                        .store(val as *const _ as *mut _, Ordering::Relaxed);

                    return (false, addr);
                }

                // Continue searching
                curr = next_link;
                next = next_link.next.load(Ordering::Relaxed);
            }

            // No next, write link to last position in list
            curr.next
                .store(val as *const _ as *mut _, Ordering::Relaxed);

            (false, addr)
        })
    }
}

/// A link in the linked list.
pub struct Link<T> {
    pub(crate) val: T,
    next: AtomicPtr<Link<T>>,
    _up: PhantomPinned,
}

impl<T> Link<T> {
    /// Create a new link.
    pub const fn new(val: T) -> Self {
        Self {
            val,
            next: AtomicPtr::new(core::ptr::null_mut()),
            _up: PhantomPinned,
        }
    }
}
