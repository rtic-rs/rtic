//! An intrusive sorted priority linked list, designed for use in `Future`s in RTIC.
use core::cmp::Ordering;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

/// Marker for Min sorted [`IntrusiveSortedLinkedList`].
pub struct Min;

/// Marker for Max sorted [`IntrusiveSortedLinkedList`].
pub struct Max;

/// The linked list kind: min-list or max-list
pub trait Kind: private::Sealed {
    #[doc(hidden)]
    fn ordering() -> Ordering;
}

impl Kind for Min {
    fn ordering() -> Ordering {
        Ordering::Less
    }
}

impl Kind for Max {
    fn ordering() -> Ordering {
        Ordering::Greater
    }
}

/// Sealed traits
mod private {
    pub trait Sealed {}
}

impl private::Sealed for Max {}
impl private::Sealed for Min {}

/// A node in the [`IntrusiveSortedLinkedList`].
pub struct Node<T> {
    pub val: T,
    next: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(val: T) -> Self {
        Self { val, next: None }
    }
}

/// The linked list.
pub struct IntrusiveSortedLinkedList<'a, T, K> {
    head: Option<NonNull<Node<T>>>,
    _kind: PhantomData<K>,
    _lt: PhantomData<&'a ()>,
}

impl<'a, T, K> fmt::Debug for IntrusiveSortedLinkedList<'a, T, K>
where
    T: Ord + core::fmt::Debug,
    K: Kind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut l = f.debug_list();
        let mut current = self.head;

        while let Some(head) = current {
            let head = unsafe { head.as_ref() };
            current = head.next;

            l.entry(&head.val);
        }

        l.finish()
    }
}

impl<'a, T, K> IntrusiveSortedLinkedList<'a, T, K>
where
    T: Ord,
    K: Kind,
{
    pub const fn new() -> Self {
        Self {
            head: None,
            _kind: PhantomData,
            _lt: PhantomData,
        }
    }

    // Push to the list.
    pub fn push(&mut self, new: &'a mut Node<T>) {
        unsafe {
            if let Some(head) = self.head {
                if head.as_ref().val.cmp(&new.val) != K::ordering() {
                    // This is newer than head, replace head
                    new.next = self.head;
                    self.head = Some(NonNull::new_unchecked(new));
                } else {
                    // It's not head, search the list for the correct placement
                    let mut current = head;

                    while let Some(next) = current.as_ref().next {
                        if next.as_ref().val.cmp(&new.val) != K::ordering() {
                            break;
                        }

                        current = next;
                    }

                    new.next = current.as_ref().next;
                    current.as_mut().next = Some(NonNull::new_unchecked(new));
                }
            } else {
                // List is empty, place at head
                self.head = Some(NonNull::new_unchecked(new))
            }
        }
    }

    /// Get an iterator over the sorted list.
    pub fn iter(&self) -> Iter<'_, T, K> {
        Iter {
            _list: self,
            index: self.head,
        }
    }

    /// Find an element in the list that can be changed and resorted.
    pub fn find_mut<F>(&mut self, mut f: F) -> Option<FindMut<'_, 'a, T, K>>
    where
        F: FnMut(&T) -> bool,
    {
        let head = self.head?;

        // Special-case, first element
        if f(&unsafe { head.as_ref() }.val) {
            return Some(FindMut {
                is_head: true,
                prev_index: None,
                index: self.head,
                list: self,
                maybe_changed: false,
            });
        }

        let mut current = head;

        while let Some(next) = unsafe { current.as_ref() }.next {
            if f(&unsafe { next.as_ref() }.val) {
                return Some(FindMut {
                    is_head: false,
                    prev_index: Some(current),
                    index: Some(next),
                    list: self,
                    maybe_changed: false,
                });
            }

            current = next;
        }

        None
    }

    /// Peek at the first element.
    pub fn peek(&self) -> Option<&T> {
        self.head.map(|head| unsafe { &head.as_ref().val })
    }

    /// Pops the first element in the list.
    ///
    /// Complexity is worst-case `O(1)`.
    pub fn pop(&mut self) -> Option<&'a Node<T>> {
        if let Some(head) = self.head {
            let v = unsafe { head.as_ref() };
            self.head = v.next;
            Some(v)
        } else {
            None
        }
    }

    /// Checks if the linked list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

/// Iterator for the linked list.
pub struct Iter<'a, T, K>
where
    T: Ord,
    K: Kind,
{
    _list: &'a IntrusiveSortedLinkedList<'a, T, K>,
    index: Option<NonNull<Node<T>>>,
}

impl<'a, T, K> Iterator for Iter<'a, T, K>
where
    T: Ord,
    K: Kind,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index?;

        let node = unsafe { index.as_ref() };
        self.index = node.next;

        Some(&node.val)
    }
}

/// Comes from [`IntrusiveSortedLinkedList::find_mut`].
pub struct FindMut<'a, 'b, T, K>
where
    T: Ord + 'b,
    K: Kind,
{
    list: &'a mut IntrusiveSortedLinkedList<'b, T, K>,
    is_head: bool,
    prev_index: Option<NonNull<Node<T>>>,
    index: Option<NonNull<Node<T>>>,
    maybe_changed: bool,
}

impl<'a, 'b, T, K> FindMut<'a, 'b, T, K>
where
    T: Ord,
    K: Kind,
{
    unsafe fn pop_internal(&mut self) -> &'b mut Node<T> {
        if self.is_head {
            // If it is the head element, we can do a normal pop
            let mut head = self.list.head.unwrap_unchecked();
            let v = head.as_mut();
            self.list.head = v.next;
            v
        } else {
            // Somewhere in the list
            let mut prev = self.prev_index.unwrap_unchecked();
            let mut curr = self.index.unwrap_unchecked();

            // Re-point the previous index
            prev.as_mut().next = curr.as_ref().next;

            curr.as_mut()
        }
    }

    /// This will pop the element from the list.
    ///
    /// Complexity is worst-case `O(1)`.
    #[inline]
    pub fn pop(mut self) -> &'b mut Node<T> {
        unsafe { self.pop_internal() }
    }

    /// This will resort the element into the correct position in the list if needed. The resorting
    /// will only happen if the element has been accessed mutably.
    ///
    /// Same as calling `drop`.
    ///
    /// Complexity is worst-case `O(N)`.
    #[inline]
    pub fn finish(self) {
        drop(self)
    }
}

impl<'b, T, K> Drop for FindMut<'_, 'b, T, K>
where
    T: Ord + 'b,
    K: Kind,
{
    fn drop(&mut self) {
        // Only resort the list if the element has changed
        if self.maybe_changed {
            unsafe {
                let val = self.pop_internal();
                self.list.push(val);
            }
        }
    }
}

impl<T, K> Deref for FindMut<'_, '_, T, K>
where
    T: Ord,
    K: Kind,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.index.unwrap_unchecked().as_ref().val }
    }
}

impl<T, K> DerefMut for FindMut<'_, '_, T, K>
where
    T: Ord,
    K: Kind,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.maybe_changed = true;
        unsafe { &mut self.index.unwrap_unchecked().as_mut().val }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_new() {
        static mut _V1: IntrusiveSortedLinkedList<u32, Max> = IntrusiveSortedLinkedList::new();
    }

    #[test]
    fn test_peek() {
        let mut ll: IntrusiveSortedLinkedList<u32, Max> = IntrusiveSortedLinkedList::new();

        let mut a = Node { val: 1, next: None };
        ll.push(&mut a);
        assert_eq!(ll.peek().unwrap(), &1);

        let mut a = Node { val: 2, next: None };
        ll.push(&mut a);
        assert_eq!(ll.peek().unwrap(), &2);

        let mut a = Node { val: 3, next: None };
        ll.push(&mut a);
        assert_eq!(ll.peek().unwrap(), &3);

        let mut ll: IntrusiveSortedLinkedList<u32, Min> = IntrusiveSortedLinkedList::new();

        let mut a = Node { val: 2, next: None };
        ll.push(&mut a);
        assert_eq!(ll.peek().unwrap(), &2);

        let mut a = Node { val: 1, next: None };
        ll.push(&mut a);
        assert_eq!(ll.peek().unwrap(), &1);

        let mut a = Node { val: 3, next: None };
        ll.push(&mut a);
        assert_eq!(ll.peek().unwrap(), &1);
    }

    #[test]
    fn test_empty() {
        let ll: IntrusiveSortedLinkedList<u32, Max> = IntrusiveSortedLinkedList::new();

        assert!(ll.is_empty())
    }

    #[test]
    fn test_updating() {
        let mut ll: IntrusiveSortedLinkedList<u32, Max> = IntrusiveSortedLinkedList::new();

        let mut a = Node { val: 1, next: None };
        ll.push(&mut a);

        let mut a = Node { val: 2, next: None };
        ll.push(&mut a);

        let mut a = Node { val: 3, next: None };
        ll.push(&mut a);

        let mut find = ll.find_mut(|v| *v == 2).unwrap();

        *find += 1000;
        find.finish();

        assert_eq!(ll.peek().unwrap(), &1002);

        let mut find = ll.find_mut(|v| *v == 3).unwrap();

        *find += 1000;
        find.finish();

        assert_eq!(ll.peek().unwrap(), &1003);

        // Remove largest element
        ll.find_mut(|v| *v == 1003).unwrap().pop();

        assert_eq!(ll.peek().unwrap(), &1002);
    }

    #[test]
    fn test_updating_1() {
        let mut ll: IntrusiveSortedLinkedList<u32, Max> = IntrusiveSortedLinkedList::new();

        let mut a = Node { val: 1, next: None };
        ll.push(&mut a);

        let v = ll.pop().unwrap();

        assert_eq!(v.val, 1);
    }

    #[test]
    fn test_updating_2() {
        let mut ll: IntrusiveSortedLinkedList<u32, Max> = IntrusiveSortedLinkedList::new();

        let mut a = Node { val: 1, next: None };
        ll.push(&mut a);

        let mut find = ll.find_mut(|v| *v == 1).unwrap();

        *find += 1000;
        find.finish();

        assert_eq!(ll.peek().unwrap(), &1001);
    }
}
