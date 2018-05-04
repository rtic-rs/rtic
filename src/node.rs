use core::cmp::Ordering;
use core::{mem, ptr};

use instant::Instant;

#[doc(hidden)]
#[repr(C)]
pub struct Node<T>
where
    T: 'static,
{
    #[cfg(feature = "timer-queue")]
    baseline: Instant,
    payload: T,
}

#[cfg(feature = "timer-queue")]
impl<T> Eq for Node<T> {}

#[cfg(feature = "timer-queue")]
impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        self.baseline == other.baseline
    }
}

#[cfg(feature = "timer-queue")]
impl<T> Ord for Node<T> {
    fn cmp(&self, other: &Node<T>) -> Ordering {
        self.baseline.cmp(&other.baseline)
    }
}

#[cfg(feature = "timer-queue")]
impl<T> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Node<T>) -> Option<Ordering> {
        Some(self.cmp(other))
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
    #[cfg(feature = "timer-queue")]
    pub fn write(self, bl: Instant, data: T) -> Payload<T> {
        self.node.baseline = bl;
        unsafe { ptr::write(&mut self.node.payload, data) }
        Payload { node: self.node }
    }

    #[cfg(not(feature = "timer-queue"))]
    pub fn write(self, data: T) -> Payload<T> {
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
    #[cfg(feature = "timer-queue")]
    pub fn read(self) -> (Instant, T, Slot<T>) {
        let data = unsafe { ptr::read(&self.node.payload) };
        (self.node.baseline, data, Slot { node: self.node })
    }

    #[cfg(not(feature = "timer-queue"))]
    pub fn read(self) -> (T, Slot<T>) {
        let data = unsafe { ptr::read(&self.node.payload) };
        (data, Slot { node: self.node })
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

    #[cfg(feature = "timer-queue")]
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

#[cfg(feature = "timer-queue")]
impl<T> Eq for TaggedPayload<T>
where
    T: Copy,
{
}

#[cfg(feature = "timer-queue")]
impl<T> Ord for TaggedPayload<T>
where
    T: Copy,
{
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.payload.node.cmp(&rhs.payload.node)
    }
}

#[cfg(feature = "timer-queue")]
impl<T> PartialEq for TaggedPayload<T>
where
    T: Copy,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.payload.node.eq(&rhs.payload.node)
    }
}

#[cfg(feature = "timer-queue")]
impl<T> PartialOrd for TaggedPayload<T>
where
    T: Copy,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
