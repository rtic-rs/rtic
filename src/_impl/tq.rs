use core::cmp::{self, Ordering};

use cortex_m::peripheral::{SCB, SYST};
use heapless::binary_heap::{BinaryHeap, Min};
use heapless::ArrayLength;
use typenum::{Max, Unsigned};

use _impl::Instant;
use resource::{Priority, Resource};

pub struct NotReady<T> {
    pub scheduled_time: Instant,
    pub index: u8,
    pub task: T,
}

impl<T> Eq for NotReady<T> {}

impl<T> Ord for NotReady<T> {
    fn cmp(&self, other: &NotReady<T>) -> Ordering {
        self.scheduled_time.cmp(&other.scheduled_time)
    }
}

impl<T> PartialEq for NotReady<T> {
    fn eq(&self, other: &NotReady<T>) -> bool {
        self.scheduled_time == other.scheduled_time
    }
}

impl<T> PartialOrd for NotReady<T> {
    fn partial_cmp(&self, other: &NotReady<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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
    pub const fn new(syst: SYST) -> Self {
        TimerQueue {
            syst,
            queue: BinaryHeap::new(),
        }
    }

    #[inline]
    pub unsafe fn enqueue(&mut self, m: NotReady<T>) {
        let mut is_empty = true;
        if self.queue
            .peek()
            .map(|head| {
                is_empty = false;
                m.scheduled_time < head.scheduled_time
            })
            .unwrap_or(true)
        {
            if is_empty {
                self.syst.enable_interrupt();
            }

            // set SysTick pending
            (*SCB::ptr()).icsr.write(1 << 26);
        }

        self.queue.push_unchecked(m);
    }
}

pub fn dispatch<T, TQ, N, F, P>(t: &mut Priority<P>, tq: &mut TQ, mut f: F)
where
    F: FnMut(&mut Priority<P>, T, u8),
    N: 'static + ArrayLength<NotReady<T>>,
    P: Max<TQ::Ceiling> + Unsigned,
    T: 'static + Copy + Send,
    TQ: Resource<Data = TimerQueue<T, N>>,
    TQ::Ceiling: Unsigned,
{
    loop {
        let next = tq.claim_mut(t, |tq, _| {
            if let Some(st) = tq.queue.peek().map(|p| p.scheduled_time) {
                let diff = st - Instant::now();

                if diff < 0 {
                    // became ready
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
