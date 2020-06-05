use core::{
    cmp::{self, Ordering},
    convert::TryFrom,
    mem,
    marker::PhantomData,
};

use cortex_m::peripheral::{SCB, SYST};
use heapless::{binary_heap::Min, ArrayLength, BinaryHeap};

use crate::time::{self, instant::Instant, time_units::*, Duration};
use crate::{Monotonic};

pub struct TimerQueue<SysTimer, Clock, Task, N>(pub BinaryHeap<NotReady<Clock, Task>, N, Min>, pub PhantomData<SysTimer>)
    where
        SysTimer: time::Clock,
        Clock: Monotonic,
        N: ArrayLength<NotReady<Clock, Task>>,
        Task: Copy;

impl<SysTimer, Clock, Task, N> TimerQueue<SysTimer, Clock, Task, N>
    where
        SysTimer: time::Clock,
        Clock: Monotonic,
        N: ArrayLength<NotReady<Clock, Task>>,
        Task: Copy,
{
    #[inline]
    pub unsafe fn enqueue_unchecked(&mut self, nr: NotReady<Clock, Task>) {
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
    pub fn dequeue(&mut self) -> Option<(Task, u8)> {
        unsafe {
            if let Some(instant) = self.0.peek().map(|p| p.instant) {
                let now = Clock::now();

                if instant < now {
                    // task became ready
                    let nr = self.0.pop_unchecked();

                    Some((nr.task, nr.index))
                } else {
                    // set a new timeout
                    const MAX: u32 = 0x00ffffff;

                    let dur: Microseconds<i64> = instant.duration_since(&now).unwrap();
                    let systick_ticks: i64 = dur
                        .into_ticks(SysTimer::PERIOD)
                        .expect("into_ticks failed");

                    // ARM Architecture Reference Manual says:
                    // "Setting SYST_RVR to zero has the effect of
                    // disabling the SysTick counter independently
                    // of the counter enable bit."
                    let systick_ticks = u32::try_from(systick_ticks).ok()?;

                    let systick_ticks = cmp::min(cmp::max(systick_ticks, 1), MAX);

                    mem::transmute::<_, SYST>(()).set_reload(systick_ticks);

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

pub struct NotReady<Clock, Task>
where
    Task: Copy,
    Clock: Monotonic,
{
    pub index: u8,
    pub instant: Instant<Clock>,
    pub task: Task,
}

impl<Clock, Task> Eq for NotReady<Clock, Task>
where
    Task: Copy,
    Clock: Monotonic,
{
}

impl<Clock, Task> Ord for NotReady<Clock, Task>
where
    Task: Copy,
    Clock: Monotonic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<Clock, Task> PartialEq for NotReady<Clock, Task>
where
    Task: Copy,
    Clock: Monotonic,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<Clock, Task> PartialOrd for NotReady<Clock, Task>
where
    Task: Copy,
    Clock: Monotonic,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
