use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LinkedIndex(u16);

impl LinkedIndex {
    #[inline]
    const unsafe fn new_unchecked(value: u16) -> Self {
        LinkedIndex(value)
    }

    #[inline]
    const fn none() -> Self {
        LinkedIndex(u16::MAX)
    }

    #[inline]
    const fn option(self) -> Option<u16> {
        if self.0 == u16::MAX {
            None
        } else {
            Some(self.0)
        }
    }
}

/// A node in the linked list.
pub struct Node<T> {
    val: MaybeUninit<T>,
    next: LinkedIndex,
}

/// Iterator for the linked list.
pub struct Iter<'a, T, Kind, const N: usize>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    list: &'a LinkedList<T, Kind, N>,
    index: LinkedIndex,
}

impl<'a, T, Kind, const N: usize> Iterator for Iter<'a, T, Kind, N>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index.option()?;

        let node = self.list.node_at(index as usize);
        self.index = node.next;

        Some(self.list.read_data_in_node_at(index as usize))
    }
}

/// Comes from [`LinkedList::find_mut`].
pub struct FindMut<'a, T, Kind, const N: usize>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    list: &'a mut LinkedList<T, Kind, N>,
    is_head: bool,
    prev_index: LinkedIndex,
    index: LinkedIndex,
    maybe_changed: bool,
}

impl<'a, T, Kind, const N: usize> FindMut<'a, T, Kind, N>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    fn pop_internal(&mut self) -> T {
        if self.is_head {
            // If it is the head element, we can do a normal pop
            unsafe { self.list.pop_unchecked() }
        } else {
            // Somewhere in the list

            // Re-point the previous index
            self.list.node_at_mut(self.prev_index.0 as usize).next =
                self.list.node_at_mut(self.index.0 as usize).next;

            // Release the index into the free queue
            self.list.node_at_mut(self.index.0 as usize).next = self.list.free;
            self.list.free = self.index;

            self.list.extract_data_in_node_at(self.index.0 as usize)
        }
    }

    /// This will pop the element from the list.
    ///
    /// Complexity is O(1).
    #[inline]
    pub fn pop(mut self) -> T {
        self.pop_internal()
    }

    /// This will resort the element into the correct position in the list in needed.
    /// Same as calling `drop`.
    ///
    /// Complexity is worst-case O(N).
    #[inline]
    pub fn finish(self) {
        drop(self)
    }
}

impl<T, Kind, const N: usize> Drop for FindMut<'_, T, Kind, N>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    fn drop(&mut self) {
        // Only resort the list if the element has changed
        if self.maybe_changed {
            let val = self.pop_internal();
            unsafe { self.list.push_unchecked(val) };
        }
    }
}

impl<T, Kind, const N: usize> Deref for FindMut<'_, T, Kind, N>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.list.read_data_in_node_at(self.index.0 as usize)
    }
}

impl<T, Kind, const N: usize> DerefMut for FindMut<'_, T, Kind, N>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.maybe_changed = true;
        self.list.read_mut_data_in_node_at(self.index.0 as usize)
    }
}

impl<T, Kind, const N: usize> fmt::Debug for FindMut<'_, T, Kind, N>
where
    T: PartialEq + PartialOrd + core::fmt::Debug,
    Kind: kind::Kind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FindMut")
            .field("prev_index", &self.prev_index)
            .field("index", &self.index)
            .field(
                "prev_value",
                &self
                    .list
                    .read_data_in_node_at(self.prev_index.option().unwrap() as usize),
            )
            .field(
                "value",
                &self
                    .list
                    .read_data_in_node_at(self.index.option().unwrap() as usize),
            )
            .finish()
    }
}

/// The linked list.
pub struct LinkedList<T, Kind, const N: usize>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    list: MaybeUninit<[Node<T>; N]>,
    head: LinkedIndex,
    free: LinkedIndex,
    _kind: PhantomData<Kind>,
}

impl<T, Kind, const N: usize> LinkedList<T, Kind, N>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn node_at(&self, index: usize) -> &Node<T> {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { &*(self.list.as_ptr() as *const Node<T>).add(index) }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn node_at_mut(&mut self, index: usize) -> &mut Node<T> {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { &mut *(self.list.as_mut_ptr() as *mut Node<T>).add(index) }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn write_data_in_node_at(&mut self, index: usize, data: T) {
        unsafe {
            self.node_at_mut(index).val.as_mut_ptr().write(data);
        }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn read_data_in_node_at(&self, index: usize) -> &T {
        unsafe { &*self.node_at(index).val.as_ptr() }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn read_mut_data_in_node_at(&mut self, index: usize) -> &mut T {
        unsafe { &mut *self.node_at_mut(index).val.as_mut_ptr() }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn extract_data_in_node_at(&mut self, index: usize) -> T {
        unsafe { self.node_at(index).val.as_ptr().read() }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    /// Safety: This can overwrite existing allocated nodes if used improperly, meaning their
    /// `Drop` methods won't run.
    #[inline]
    unsafe fn write_node_at(&mut self, index: usize, node: Node<T>) {
        (self.list.as_mut_ptr() as *mut Node<T>)
            .add(index)
            .write(node)
    }

    /// Create a new linked list.
    pub fn new() -> Self {
        let mut list = LinkedList {
            list: MaybeUninit::uninit(),
            head: LinkedIndex::none(),
            free: unsafe { LinkedIndex::new_unchecked(0) },
            _kind: PhantomData,
        };

        let len = N as u16;
        let mut free = 0;

        if len == 0 {
            list.free = LinkedIndex::none();
            return list;
        }

        // Initialize indexes
        while free < len - 1 {
            unsafe {
                list.write_node_at(
                    free as usize,
                    Node {
                        val: MaybeUninit::uninit(),
                        next: LinkedIndex::new_unchecked(free + 1),
                    },
                );
            }
            free += 1;
        }

        // Initialize final index
        unsafe {
            list.write_node_at(
                free as usize,
                Node {
                    val: MaybeUninit::uninit(),
                    next: LinkedIndex::none(),
                },
            );
        }

        list
    }

    /// Push unchecked
    ///
    /// Complexity is O(N).
    ///
    /// # Safety
    ///
    /// Assumes that the list is not full.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        let new = self.free.0;
        // Store the data and update the next free spot
        self.write_data_in_node_at(new as usize, value);
        self.free = self.node_at(new as usize).next;

        if let Some(head) = self.head.option() {
            // Check if we need to replace head
            if self
                .read_data_in_node_at(head as usize)
                .partial_cmp(self.read_data_in_node_at(new as usize))
                != Kind::ordering()
            {
                self.node_at_mut(new as usize).next = self.head;
                self.head = LinkedIndex::new_unchecked(new);
            } else {
                // It's not head, search the list for the correct placement
                let mut current = head;

                while let Some(next) = self.node_at(current as usize).next.option() {
                    if self
                        .read_data_in_node_at(next as usize)
                        .partial_cmp(self.read_data_in_node_at(new as usize))
                        != Kind::ordering()
                    {
                        break;
                    }

                    current = next;
                }

                self.node_at_mut(new as usize).next = self.node_at(current as usize).next;
                self.node_at_mut(current as usize).next = LinkedIndex::new_unchecked(new);
            }
        } else {
            self.node_at_mut(new as usize).next = self.head;
            self.head = LinkedIndex::new_unchecked(new);
        }
    }

    /// Pushes an element to the linked list and sorts it into place.
    ///
    /// Complexity is O(N).
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if !self.is_full() {
            Ok(unsafe { self.push_unchecked(value) })
        } else {
            Err(value)
        }
    }

    /// Get an iterator over the sorted list.
    pub fn iter(&self) -> Iter<'_, T, Kind, N> {
        Iter {
            list: self,
            index: self.head,
        }
    }

    /// Find an element in the list.
    pub fn find_mut<F>(&mut self, mut f: F) -> Option<FindMut<'_, T, Kind, N>>
    where
        F: FnMut(&T) -> bool,
    {
        let head = self.head.option()?;

        // Special-case, first element
        if f(self.read_data_in_node_at(head as usize)) {
            return Some(FindMut {
                is_head: true,
                prev_index: LinkedIndex::none(),
                index: self.head,
                list: self,
                maybe_changed: false,
            });
        }

        let mut current = head;

        while let Some(next) = self.node_at(current as usize).next.option() {
            if f(self.read_data_in_node_at(next as usize)) {
                return Some(FindMut {
                    is_head: false,
                    prev_index: unsafe { LinkedIndex::new_unchecked(current) },
                    index: unsafe { LinkedIndex::new_unchecked(next) },
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
        self.head
            .option()
            .map(|head| self.read_data_in_node_at(head as usize))
    }

    /// Pop unchecked
    ///
    /// # Safety
    ///
    /// Assumes that the list is not empty.
    pub unsafe fn pop_unchecked(&mut self) -> T {
        let head = self.head.0;
        let current = head;
        self.head = self.node_at(head as usize).next;
        self.node_at_mut(current as usize).next = self.free;
        self.free = LinkedIndex::new_unchecked(current);

        self.extract_data_in_node_at(current as usize)
    }

    /// Pops the first element in the list.
    ///
    /// Complexity is O(1).
    pub fn pop(&mut self) -> Result<T, ()> {
        if !self.is_empty() {
            Ok(unsafe { self.pop_unchecked() })
        } else {
            Err(())
        }
    }

    /// Checks if the linked list is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.free.option().is_none()
    }

    /// Checks if the linked list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head.option().is_none()
    }
}

impl<T, Kind, const N: usize> Drop for LinkedList<T, Kind, N>
where
    T: PartialEq + PartialOrd,
    Kind: kind::Kind,
{
    fn drop(&mut self) {
        let mut index = self.head;

        while let Some(i) = index.option() {
            let node = self.node_at_mut(i as usize);
            index = node.next;

            unsafe {
                ptr::drop_in_place(node.val.as_mut_ptr());
            }
        }
    }
}

impl<T, Kind, const N: usize> fmt::Debug for LinkedList<T, Kind, N>
where
    T: PartialEq + PartialOrd + core::fmt::Debug,
    Kind: kind::Kind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// Min sorted linked list.
pub struct Min;

/// Max sorted linked list.
pub struct Max;

/// Sealed traits and implementations for `linked_list`
pub mod kind {
    use super::{Max, Min};
    use core::cmp::Ordering;

    /// The linked list kind: min first or max first
    pub unsafe trait Kind {
        #[doc(hidden)]
        fn ordering() -> Option<Ordering>;
    }

    unsafe impl Kind for Min {
        #[inline]
        fn ordering() -> Option<Ordering> {
            Some(Ordering::Less)
        }
    }

    unsafe impl Kind for Max {
        #[inline]
        fn ordering() -> Option<Ordering> {
            Some(Ordering::Greater)
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_peek() {
        let mut ll: LinkedList<u32, Max, 3> = LinkedList::new();

        ll.push(1).unwrap();
        assert_eq!(ll.peek().unwrap(), &1);

        ll.push(2).unwrap();
        assert_eq!(ll.peek().unwrap(), &2);

        ll.push(3).unwrap();
        assert_eq!(ll.peek().unwrap(), &3);

        let mut ll: LinkedList<u32, Min, 3> = LinkedList::new();

        ll.push(2).unwrap();
        assert_eq!(ll.peek().unwrap(), &2);

        ll.push(1).unwrap();
        assert_eq!(ll.peek().unwrap(), &1);

        ll.push(3).unwrap();
        assert_eq!(ll.peek().unwrap(), &1);
    }

    #[test]
    fn test_full() {
        let mut ll: LinkedList<u32, Max, 3> = LinkedList::new();
        ll.push(1).unwrap();
        ll.push(2).unwrap();
        ll.push(3).unwrap();

        assert!(ll.is_full())
    }

    #[test]
    fn test_empty() {
        let ll: LinkedList<u32, Max, 3> = LinkedList::new();

        assert!(ll.is_empty())
    }

    #[test]
    fn test_zero_size() {
        let ll: LinkedList<u32, Max, 0> = LinkedList::new();

        assert!(ll.is_empty());
        assert!(ll.is_full());
    }

    #[test]
    fn test_rejected_push() {
        let mut ll: LinkedList<u32, Max, 3> = LinkedList::new();
        ll.push(1).unwrap();
        ll.push(2).unwrap();
        ll.push(3).unwrap();

        // This won't fit
        let r = ll.push(4);

        assert_eq!(r, Err(4));
    }

    #[test]
    fn test_updating() {
        let mut ll: LinkedList<u32, Max, 3> = LinkedList::new();
        ll.push(1).unwrap();
        ll.push(2).unwrap();
        ll.push(3).unwrap();

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
}
