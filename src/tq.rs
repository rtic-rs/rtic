use core::cmp::{self, Ordering};

use cortex_m::peripheral::{SCB, SYST};
use heapless::binary_heap::{BinaryHeap, Min};
use heapless::ArrayLength;
use typenum::{Max, Maximum, Unsigned};

use instant::Instant;
use resource::{Resource, Threshold};

pub struct Message<T> {
    pub baseline: Instant,
    pub index: u8,
    pub task: T,
}

impl<T> Eq for Message<T> {}

impl<T> Ord for Message<T> {
    fn cmp(&self, other: &Message<T>) -> Ordering {
        self.baseline.cmp(&other.baseline)
    }
}

impl<T> PartialEq for Message<T> {
    fn eq(&self, other: &Message<T>) -> bool {
        self.baseline == other.baseline
    }
}

impl<T> PartialOrd for Message<T> {
    fn partial_cmp(&self, other: &Message<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[doc(hidden)]
pub struct TimerQueue<T, N>
where
    N: ArrayLength<Message<T>>,
    T: Copy,
{
    pub syst: SYST,
    pub queue: BinaryHeap<Message<T>, N, Min>,
}

impl<T, N> TimerQueue<T, N>
where
    N: ArrayLength<Message<T>>,
    T: Copy,
{
    pub const fn new(syst: SYST) -> Self {
        TimerQueue {
            syst,
            queue: BinaryHeap::new(),
        }
    }

    #[inline]
    pub unsafe fn enqueue(&mut self, m: Message<T>) {
        let mut is_empty = true;
        if self.queue
            .peek()
            .map(|head| {
                is_empty = false;
                m.baseline < head.baseline
            })
            .unwrap_or(true)
        {
            if is_empty {
                self.syst.enable_interrupt();
            }

            // set SysTick pending
            unsafe { (*SCB::ptr()).icsr.write(1 << 26) }
        }

        self.queue.push_unchecked(m);
    }
}

pub fn dispatch<T, TQ, N, F, P>(t: &mut Threshold<P>, tq: &mut TQ, mut f: F)
where
    F: FnMut(&mut Threshold<P>, T, u8),
    N: 'static + ArrayLength<Message<T>>,
    P: Max<TQ::Ceiling> + Unsigned,
    T: 'static + Copy + Send,
    TQ: Resource<Data = TimerQueue<T, N>>,
    TQ::Ceiling: Unsigned,
{
    loop {
        let next = tq.claim_mut(t, |tq, _| {
            if let Some(bl) = tq.queue.peek().map(|p| p.baseline) {
                let diff = bl - Instant::now();

                if diff < 0 {
                    // message ready
                    let m = unsafe { tq.queue.pop_unchecked() };

                    Some((m.task, m.index))
                } else {
                    // set a new timeout
                    const MAX: u32 = 0x00ffffff;

                    tq.syst.set_reload(cmp::min(MAX, diff as u32));

                    // start counting from the new reload
                    tq.syst.clear_current();

                    None
                }
            } else {
                // empty queue
                tq.syst.disable_interrupt();
                None
            }
        });

        if let Some((task, index)) = next {
            f(t, task, index)
        } else {
            return;
        }
    }
}
