use core::cmp::Ordering;
use core::marker::Unsize;
use core::ptr;

use cortex_m::peripheral::SYST;
use heapless::binary_heap::{BinaryHeap, Min};
pub use heapless::ring_buffer::{Consumer, Producer, RingBuffer};
use untagged_option::UntaggedOption;

pub struct TimerQueue<T, A>
where
    A: Unsize<[Message<T>]>,
{
    pub syst: SYST,
    pub queue: BinaryHeap<Message<T>, A, Min>,
}

impl<T, A> TimerQueue<T, A>
where
    A: Unsize<[Message<T>]>,
{
    pub fn new(syst: SYST) -> Self {
        TimerQueue {
            syst,
            queue: BinaryHeap::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Message<T> {
    pub baseline: u32,
    pub task: T,
    pub payload: usize,
}

impl<T> Message<T> {
    pub fn new<P>(bl: u32, task: T, payload: Payload<P>) -> Self {
        Message {
            baseline: bl,
            task,
            payload: payload.erase(),
        }
    }
}

impl<T> PartialEq for Message<T> {
    fn eq(&self, other: &Self) -> bool {
        self.baseline.eq(&other.baseline)
    }
}

impl<T> Eq for Message<T> {}

impl<T> PartialOrd for Message<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Message<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.baseline as i32)
            .wrapping_sub(other.baseline as i32)
            .cmp(&0)
    }
}

pub struct Node<T>
where
    T: 'static,
{
    data: UntaggedOption<T>,
    next: Option<&'static mut Node<T>>,
}

impl<T> Node<T> {
    pub const fn new() -> Self {
        Node {
            data: UntaggedOption::none(),
            next: None,
        }
    }
}

pub struct Payload<T>
where
    T: 'static,
{
    node: &'static mut Node<T>,
}

impl<T> Payload<T> {
    pub unsafe fn from(ptr: usize) -> Self {
        Payload {
            node: &mut *(ptr as *mut _),
        }
    }

    pub fn erase(self) -> usize {
        self.node as *mut _ as usize
    }

    pub fn read(self) -> (T, Slot<T>) {
        unsafe {
            let payload = ptr::read(&self.node.data.some);

            (payload, Slot::new(self.node))
        }
    }
}

pub struct Slot<T>
where
    T: 'static,
{
    node: &'static mut Node<T>,
}

impl<T> Slot<T> {
    pub fn new(node: &'static mut Node<T>) -> Self {
        Slot { node }
    }

    pub fn write(self, data: T) -> Payload<T> {
        unsafe {
            ptr::write(&mut self.node.data.some, data);
            Payload { node: self.node }
        }
    }
}

pub struct FreeList<T>
where
    T: 'static,
{
    head: Option<Slot<T>>,
}

impl<T> FreeList<T> {
    pub const fn new() -> Self {
        FreeList { head: None }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn pop(&mut self) -> Option<Slot<T>> {
        self.head.take().map(|head| {
            self.head = head.node.next.take().map(Slot::new);
            head
        })
    }

    pub fn push(&mut self, free: Slot<T>) {
        free.node.next = self.head.take().map(|slot| slot.node);
        self.head = Some(Slot::new(free.node));
    }
}
