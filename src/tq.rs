use core::cmp::{self, Ordering};

use cortex_m::peripheral::{SCB, SYST};
use heapless::{binary_heap::Min, ArrayLength, BinaryHeap};

use crate::{Instant, Mutex};

pub struct TimerQueue<T, N>
where
    N: ArrayLength<NotReady<T>>,
    T: Copy,
{
    pub syst: SYST,
    pub queue: BinaryHeap<NotReady<T>, N, Min>,
}

impl<T, N> TimerQueue<T, N>
where
    N: ArrayLength<NotReady<T>>,
    T: Copy,
{
    pub fn new(syst: SYST) -> Self {
        TimerQueue {
            syst,
            queue: BinaryHeap::new(),
        }
    }

    #[inline]
    pub unsafe fn enqueue_unchecked(&mut self, nr: NotReady<T>) {
        let mut is_empty = true;
        if self
            .queue
            .peek()
            .map(|head| {
                is_empty = false;
                nr.instant < head.instant
            })
            .unwrap_or(true)
        {
            if is_empty {
                self.syst.enable_interrupt();
            }

            // set SysTick pending
            (*SCB::ptr()).icsr.write(1 << 26);
        }

        self.queue.push_unchecked(nr);
    }
}

pub struct NotReady<T>
where
    T: Copy,
{
    pub index: u8,
    pub instant: Instant,
    pub task: T,
}

impl<T> Eq for NotReady<T> where T: Copy {}

impl<T> Ord for NotReady<T>
where
    T: Copy,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<T> PartialEq for NotReady<T>
where
    T: Copy,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<T> PartialOrd for NotReady<T>
where
    T: Copy,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

#[inline(always)]
pub fn isr<TQ, T, N, F>(mut tq: TQ, mut f: F)
where
    TQ: Mutex<Data = TimerQueue<T, N>>,
    T: Copy + Send,
    N: ArrayLength<NotReady<T>>,
    F: FnMut(T, u8),
{
    loop {
        // XXX does `#[inline(always)]` improve performance or not?
        let next = tq.lock(#[inline(always)]
        |tq| {
            if let Some(instant) = tq.queue.peek().map(|p| p.instant) {
                let diff = instant.0.wrapping_sub(Instant::now().0);

                if diff < 0 {
                    // task became ready
                    let m = unsafe { tq.queue.pop_unchecked() };

                    Some((m.task, m.index))
                } else {
                    // set a new timeout
                    const MAX: u32 = 0x00ffffff;

                    tq.syst.set_reload(cmp::min(MAX, diff as u32));

                    // start counting down from the new reload
                    tq.syst.clear_current();

                    None
                }
            } else {
                // the queue is empty
                tq.syst.disable_interrupt();
                None
            }
        });

        if let Some((task, index)) = next {
            f(task, index)
        } else {
            return;
        }
    }
}
