use core::{
    cmp::{self, Ordering},
    convert::TryInto,
    mem,
    ops::Sub,
};

use cortex_m::peripheral::{SCB, SYST};
use heapless::{binary_heap::Min, ArrayLength, BinaryHeap};

use crate::Monotonic;

pub struct TimerQueue<M, T, N>(pub BinaryHeap<NotReady<M, T>, N, Min>)
where
    M: Monotonic,
    <M::Instant as Sub>::Output: TryInto<u32>,
    N: ArrayLength<NotReady<M, T>>,
    T: Copy;

impl<M, T, N> TimerQueue<M, T, N>
where
    M: Monotonic,
    <M::Instant as Sub>::Output: TryInto<u32>,
    N: ArrayLength<NotReady<M, T>>,
    T: Copy,
{
    #[inline]
    pub unsafe fn enqueue_unchecked(&mut self, nr: NotReady<M, T>) {
        let mut is_empty = true;
        if self
            .0
            .peek()
            .map(|head| {
                is_empty = false;
                nr.instant < head.instant
            })
            .unwrap_or(true)
        {
            if is_empty {
                mem::transmute::<_, SYST>(()).enable_interrupt();
            }

            // set SysTick pending
            SCB::set_pendst();
        }

        self.0.push_unchecked(nr);
    }

    #[inline]
    pub fn dequeue(&mut self) -> Option<(T, u8)> {
        unsafe {
            if let Some(instant) = self.0.peek().map(|p| p.instant) {
                let now = M::now();

                if instant < now {
                    // task became ready
                    let nr = self.0.pop_unchecked();

                    Some((nr.task, nr.index))
                } else {
                    // set a new timeout
                    const MAX: u32 = 0x00ffffff;

                    let ratio = M::ratio();
                    let dur = match (instant - now).try_into().ok().and_then(|x| {
                        x.checked_mul(ratio.numerator)
                            .map(|x| x / ratio.denominator)
                    }) {
                        None => MAX,
                        Some(x) => cmp::min(MAX, x),
                    };
                    mem::transmute::<_, SYST>(()).set_reload(dur);

                    // start counting down from the new reload
                    mem::transmute::<_, SYST>(()).clear_current();

                    None
                }
            } else {
                // the queue is empty
                mem::transmute::<_, SYST>(()).disable_interrupt();

                None
            }
        }
    }
}

pub struct NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
    <M::Instant as Sub>::Output: TryInto<u32>,
{
    pub index: u8,
    pub instant: M::Instant,
    pub task: T,
}

impl<M, T> Eq for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
    <M::Instant as Sub>::Output: TryInto<u32>,
{
}

impl<M, T> Ord for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
    <M::Instant as Sub>::Output: TryInto<u32>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<M, T> PartialEq for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
    <M::Instant as Sub>::Output: TryInto<u32>,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<M, T> PartialOrd for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
    <M::Instant as Sub>::Output: TryInto<u32>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
