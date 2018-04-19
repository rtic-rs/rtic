use core::cmp::Ordering;
use core::marker::Unsize;
use core::ptr;

use cortex_m::peripheral::{DWT, SCB, SYST};
use heapless::binary_heap::{BinaryHeap, Min};
pub use heapless::ring_buffer::{Consumer, Producer, RingBuffer};
use untagged_option::UntaggedOption;

#[derive(Clone, Copy)]
pub struct Message<T> {
    // relative to the TimerQueue baseline
    pub deadline: u32,
    pub task: T,
    pub payload: usize,
}

impl<T> Message<T> {
    fn new<P>(dl: u32, task: T, payload: Payload<P>) -> Self {
        Message {
            deadline: dl,
            task,
            payload: payload.erase(),
        }
    }
}

impl<T> PartialEq for Message<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deadline.eq(&other.deadline)
    }
}

impl<T> Eq for Message<T> {}

impl<T> PartialOrd for Message<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.deadline.partial_cmp(&other.deadline)
    }
}

impl<T> Ord for Message<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deadline.cmp(&other.deadline)
    }
}

pub struct TimerQueue<T, A>
where
    A: Unsize<[Message<T>]>,
{
    pub syst: SYST,
    pub baseline: u32,
    pub queue: BinaryHeap<Message<T>, A, Min>,
}

impl<T, A> TimerQueue<T, A>
where
    A: Unsize<[Message<T>]>,
{
    pub fn new(syst: SYST) -> Self {
        TimerQueue {
            baseline: 0,
            queue: BinaryHeap::new(),
            syst,
        }
    }

    pub fn insert<P>(
        &mut self,
        bl: u32,
        after: u32,
        task: T,
        payload: P,
        slot: Slot<P>,
    ) -> Result<(), (P, Slot<P>)> {
        if self.queue.len() == self.queue.capacity() {
            Err((payload, slot))
        } else {
            if self.queue.is_empty() {
                self.baseline = bl;
            }

            let dl = bl.wrapping_add(after).wrapping_sub(self.baseline);

            if self.queue.peek().map(|m| dl < m.deadline).unwrap_or(true) {
                // the new message is the most urgent; set a new timeout
                let now = DWT::get_cycle_count();

                if let Some(timeout) = dl.wrapping_add(self.baseline).checked_sub(now) {
                    self.syst.disable_counter();
                    self.syst.set_reload(timeout);
                    self.syst.clear_current();
                    self.syst.enable_counter();
                } else {
                    // message already expired, pend immediately
                    // NOTE(unsafe) atomic write to a stateless (from the programmer PoV) register
                    unsafe { (*SCB::ptr()).icsr.write(1 << 26) }
                }
            }

            self.queue
                .push(Message::new(dl, task, slot.write(payload)))
                .unwrap_or_else(|_| unreachable!());

            Ok(())
        }
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
