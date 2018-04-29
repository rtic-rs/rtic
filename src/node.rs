use core::cmp::Ordering;
use core::{mem, ptr};

use instant::Instant;

#[doc(hidden)]
#[repr(C)]
pub struct Node<T>
where
    T: 'static,
{
    baseline: Instant,
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

#[doc(hidden)]
pub struct Slot<T>
where
    T: 'static,
{
    node: &'static mut Node<T>,
}

impl<T> Slot<T> {
    pub fn write(self, bl: Instant, data: T) -> Payload<T> {
        self.node.baseline = bl;
        unsafe { ptr::write(&mut self.node.payload, data) }
        Payload { node: self.node }
    }
}

impl<T> Into<Slot<T>> for &'static mut Node<T> {
    fn into(self) -> Slot<T> {
        Slot { node: self }
    }
}

#[doc(hidden)]
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

#[doc(hidden)]
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
