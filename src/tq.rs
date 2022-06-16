use crate::Monotonic;
use core::cmp::Ordering;
use heapless::sorted_linked_list::{LinkedIndexU16, Min, SortedLinkedList};

pub struct TimerQueue<Mono, Task, const N: usize>(
    pub SortedLinkedList<NotReady<Mono, Task>, LinkedIndexU16, Min, N>,
)
where
    Mono: Monotonic,
    Task: Copy;

impl<Mono, Task, const N: usize> TimerQueue<Mono, Task, N>
where
    Mono: Monotonic,
    Task: Copy,
{
    /// # Safety
    ///
    /// Writing to memory with a transmute in order to enable
    /// interrupts of the ``SysTick`` timer
    ///
    /// Enqueue a task without checking if it is full
    #[inline]
    pub unsafe fn enqueue_unchecked<F1, F2>(
        &mut self,
        nr: NotReady<Mono, Task>,
        enable_interrupt: F1,
        pend_handler: F2,
        mono: Option<&mut Mono>,
    ) where
        F1: FnOnce(),
        F2: FnOnce(),
    {
        // Check if the top contains a non-empty element and if that element is
        // greater than nr
        let if_heap_max_greater_than_nr =
            self.0.peek().map_or(true, |head| nr.instant < head.instant);

        if if_heap_max_greater_than_nr {
            if Mono::DISABLE_INTERRUPT_ON_EMPTY_QUEUE && self.0.is_empty() {
                if let Some(mono) = mono {
                    mono.enable_timer();
                }
                enable_interrupt();
            }

            pend_handler();
        }

        self.0.push_unchecked(nr);
    }

    /// Check if the timer queue is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Cancel the marker value
    pub fn cancel_marker(&mut self, marker: u32) -> Option<(Task, u8)> {
        if let Some(val) = self.0.find_mut(|nr| nr.marker == marker) {
            let nr = val.pop();

            Some((nr.task, nr.index))
        } else {
            None
        }
    }

    /// Update the instant at an marker value to a new instant
    #[allow(clippy::result_unit_err)]
    pub fn update_marker<F: FnOnce()>(
        &mut self,
        marker: u32,
        new_marker: u32,
        instant: Mono::Instant,
        pend_handler: F,
    ) -> Result<(), ()> {
        if let Some(mut val) = self.0.find_mut(|nr| nr.marker == marker) {
            val.instant = instant;
            val.marker = new_marker;

            // On update pend the handler to reconfigure the next compare match
            pend_handler();

            Ok(())
        } else {
            Err(())
        }
    }

    /// Dequeue a task from the ``TimerQueue``
    pub fn dequeue<F>(&mut self, disable_interrupt: F, mono: &mut Mono) -> Option<(Task, u8)>
    where
        F: FnOnce(),
    {
        mono.clear_compare_flag();

        if let Some(instant) = self.0.peek().map(|p| p.instant) {
            if instant <= mono.now() {
                // task became ready
                let nr = unsafe { self.0.pop_unchecked() };

                Some((nr.task, nr.index))
            } else {
                // Set compare
                mono.set_compare(instant);

                // Double check that the instant we set is really in the future, else
                // dequeue. If the monotonic is fast enough it can happen that from the
                // read of now to the set of the compare, the time can overflow. This is to
                // guard against this.
                if instant <= mono.now() {
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
    pub instant: Mono::Instant,
    pub task: Task,
    pub marker: u32,
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
        Some(self.cmp(other))
    }
}
