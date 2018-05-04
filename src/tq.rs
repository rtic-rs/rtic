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

enum State<T>
where
    T: Copy,
{
    Payload { task: T, index: u8 },
    Baseline(Instant),
    Done,
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
        if self.queue
            .peek()
            .map(|head| m.baseline < head.baseline)
            .unwrap_or(true)
        {
            self.syst.enable_interrupt();
            // set SysTick pending
            unsafe { (*SCB::ptr()).icsr.write(1 << 26) }
        }

        self.queue.push_unchecked(m);
    }
}

pub fn dispatch<T, TQ, N, F, P>(t: &mut Threshold<P>, tq: &mut TQ, mut f: F)
where
    F: FnMut(&mut Threshold<P>, T, u8),
    Maximum<P, TQ::Ceiling>: Unsigned,
    N: 'static + ArrayLength<Message<T>>,
    P: Unsigned + Max<TQ::Ceiling>,
    T: 'static + Copy + Send,
    TQ: Resource<Data = TimerQueue<T, N>>,
{
    loop {
        let state = tq.claim_mut(t, |tq, _| {
            if let Some(bl) = tq.queue.peek().map(|p| p.baseline) {
                if Instant::now() >= bl {
                    // message ready
                    let m = unsafe { tq.queue.pop_unchecked() };
                    State::Payload {
                        task: m.task,
                        index: m.index,
                    }
                } else {
                    // set a new timeout
                    State::Baseline(bl)
                }
            } else {
                // empty queue
                tq.syst.disable_interrupt();
                State::Done
            }
        });

        match state {
            State::Payload { task, index } => f(t, task, index),
            State::Baseline(bl) => {
                const MAX: u32 = 0x00ffffff;

                let diff = bl - Instant::now();

                if diff < 0 {
                    // message became ready
                    continue;
                } else {
                    tq.claim_mut(t, |tq, _| {
                        tq.syst.set_reload(cmp::min(MAX, diff as u32));
                        // start counting from the new reload
                        tq.syst.clear_current();
                    });
                    return;
                }
            }
            State::Done => {
                return;
            }
        }
    }
}
