use crate::{
    time::{Clock, Instant},
    Monotonic,
};
use core::cmp::Ordering;
use heapless::{binary_heap::Min, ArrayLength, BinaryHeap};

pub struct TimerQueue<Mono, Task, N>(pub BinaryHeap<NotReady<Mono, Task>, N, Min>)
where
    Mono: Monotonic,
    N: ArrayLength<NotReady<Mono, Task>>,
    Task: Copy;

impl<Mono, Task, N> TimerQueue<Mono, Task, N>
where
    Mono: Monotonic,
    N: ArrayLength<NotReady<Mono, Task>>,
    Task: Copy,
{
    /// # Safety
    ///
    /// Writing to memory with a transmute in order to enable
    /// interrupts of the SysTick timer
    ///
    /// Enqueue a task without checking if it is full
    #[inline]
    pub unsafe fn enqueue_unchecked<F1, F2>(
        &mut self,
        nr: NotReady<Mono, Task>,
        enable_interrupt: F1,
        pend_handler: F2,
        mono: &mut Mono,
    ) where
        F1: FnOnce(),
        F2: FnOnce(),
    {
        let mut is_empty = true;
        // Check if the top contains a non-empty element and if that element is
        // greater than nr
        let if_heap_max_greater_than_nr = self
            .0
            .peek()
            .map(|head| {
                is_empty = false;
                nr.instant < head.instant
            })
            .unwrap_or(true);
        if if_heap_max_greater_than_nr {
            if Mono::DISABLE_INTERRUPT_ON_EMPTY_QUEUE && is_empty {
                // mem::transmute::<_, SYST>(()).enable_interrupt();A
                mono.enable_timer();
                enable_interrupt();
            }

            // Set SysTick pending
            // SCB::set_pendst();
            pend_handler();
        }

        self.0.push_unchecked(nr);
    }

    /// Check if the timer queue is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn unwrapper<T, E>(val: Result<T, E>) -> T {
        if let Ok(v) = val {
            v
        } else {
            unreachable!("Your monotonic is not infallible")
        }
    }

    /// Dequeue a task from the TimerQueue
    #[inline]
    pub fn dequeue<F>(&mut self, disable_interrupt: F, mono: &mut Mono) -> Option<(Task, u8)>
    where
        F: FnOnce(),
    {
        mono.clear_compare_flag();

        if let Some(instant) = self.0.peek().map(|p| p.instant) {
            if instant <= Self::unwrapper(Clock::try_now(mono)) {
                // task became ready
                let nr = unsafe { self.0.pop_unchecked() };

                Some((nr.task, nr.index))
            } else {
                // TODO: Fix this hack...
                // Extract the compare time.
                mono.set_compare(*instant.duration_since_epoch().integer());

                // Double check that the instant we set is really in the future, else
                // dequeue. If the monotonic is fast enough it can happen that from the
                // read of now to the set of the compare, the time can overflow. This is to
                // guard against this.
                if instant <= Self::unwrapper(Clock::try_now(mono)) {
                    let nr = unsafe { self.0.pop_unchecked() };

                    Some((nr.task, nr.index))
                } else {
                    None
                }
            }
        } else {
            // The queue is empty, disable the interrupt.
            if Mono::DISABLE_INTERRUPT_ON_EMPTY_QUEUE {
                disable_interrupt();
                mono.disable_timer();
            }

            None
        }
    }
}

pub struct NotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    pub index: u8,
    pub instant: Instant<Mono>,
    pub task: Task,
}

impl<Mono, Task> Eq for NotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
}

impl<Mono, Task> Ord for NotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<Mono, Task> PartialEq for NotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<Mono, Task> PartialOrd for NotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
