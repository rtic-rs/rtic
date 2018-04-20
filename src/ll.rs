use core::cmp::Ordering;
use core::marker::Unsize;
use core::ops;
use core::{mem, ptr};

use cortex_m::peripheral::{DWT, SYST};
use heapless::binary_heap::{BinaryHeap, Min};
pub use heapless::ring_buffer::{Consumer, Producer, RingBuffer};

#[repr(C)]
pub struct Node<T>
where
    T: 'static,
{
    baseline: Instant,
    next: Option<Slot<T>>,
    payload: T,
}

impl<T> Eq for Node<T> {}

impl<T> Ord for Node<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.baseline.cmp(&rhs.baseline)
    }
}

impl<T> PartialEq for Node<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.baseline.eq(&rhs.baseline)
    }
}

impl<T> PartialOrd for Node<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
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

    pub fn write(self, bl: Instant, data: T) -> Payload<T> {
        self.node.baseline = bl;
        unsafe { ptr::write(&mut self.node.payload, data) }
        Payload { node: self.node }
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

    pub fn pop(&mut self) -> Option<Slot<T>> {
        self.head.take().map(|head| {
            self.head = head.node.next.take();
            head
        })
    }

    pub fn push(&mut self, slot: Slot<T>) {
        slot.node.next = self.head.take();
        self.head = Some(slot);
    }
}

pub struct Payload<T>
where
    T: 'static,
{
    node: &'static mut Node<T>,
}

impl<T> Payload<T> {
    pub fn read(self) -> (Instant, T, Slot<T>) {
        let data = unsafe { ptr::read(&self.node.payload) };
        (self.node.baseline, data, Slot { node: self.node })
    }

    pub fn tag<A>(self, tag: A) -> TaggedPayload<A>
    where
        A: Copy,
    {
        TaggedPayload {
            tag,
            payload: unsafe { mem::transmute(self) },
        }
    }
}

pub struct TaggedPayload<A>
where
    A: Copy,
{
    tag: A,
    payload: Payload<!>,
}

impl<A> TaggedPayload<A>
where
    A: Copy,
{
    pub unsafe fn coerce<T>(self) -> Payload<T> {
        mem::transmute(self.payload)
    }

    pub fn baseline(&self) -> Instant {
        self.payload.node.baseline
    }

    pub fn tag(&self) -> A {
        self.tag
    }

    pub fn retag<B>(self, tag: B) -> TaggedPayload<B>
    where
        B: Copy,
    {
        TaggedPayload {
            tag,
            payload: self.payload,
        }
    }
}

impl<T> Eq for TaggedPayload<T>
where
    T: Copy,
{
}

impl<T> Ord for TaggedPayload<T>
where
    T: Copy,
{
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.payload.node.cmp(&rhs.payload.node)
    }
}

impl<T> PartialEq for TaggedPayload<T>
where
    T: Copy,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.payload.node.eq(&rhs.payload.node)
    }
}

impl<T> PartialOrd for TaggedPayload<T>
where
    T: Copy,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

pub struct TimerQueue<T, A>
where
    A: Unsize<[TaggedPayload<T>]>,
    T: Copy,
{
    pub syst: SYST,
    pub queue: BinaryHeap<TaggedPayload<T>, A, Min>,
}

impl<T, A> TimerQueue<T, A>
where
    A: Unsize<[TaggedPayload<T>]>,
    T: Copy,
{
    pub const fn new(syst: SYST) -> Self {
        TimerQueue {
            syst,
            queue: BinaryHeap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Instant(u32);

impl Instant {
    pub fn now() -> Self {
        Instant(DWT::get_cycle_count())
    }
}

impl Eq for Instant {}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> Ordering {
        (self.0 as i32).wrapping_sub(rhs.0 as i32).cmp(&0)
    }
}

impl PartialEq for Instant {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl ops::Add<u32> for Instant {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        Instant(self.0.wrapping_add(rhs))
    }
}

impl ops::Sub for Instant {
    type Output = i32;

    fn sub(self, rhs: Self) -> i32 {
        (self.0 as i32).wrapping_sub(rhs.0 as i32)
    }
}

pub const unsafe fn uninitialized<T>() -> T {
    #[allow(unions_with_drop_fields)]
    union U<T> {
        some: T,
        none: (),
    }

    U { none: () }.some
}
