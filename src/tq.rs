use crate::{time::Instant, Monotonic};
use core::cmp::Ordering;
use heapless::{binary_heap::Min, ArrayLength, BinaryHeap};

pub struct TimerQueue<M, T, N>(pub BinaryHeap<NotReady<M, T>, N, Min>)
where
    M: Monotonic,
    N: ArrayLength<NotReady<M, T>>,
    T: Copy;

impl<M, T, N> TimerQueue<M, T, N>
where
    M: Monotonic,
    N: ArrayLength<NotReady<M, T>>,
    T: Copy,
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
        nr: NotReady<M, T>,
        enable_interrupt: F1,
        pend_handler: F2,
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
            if is_empty {
                // mem::transmute::<_, SYST>(()).enable_interrupt();
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

    /// Dequeue a task from the TimerQueue
    #[inline]
    pub fn dequeue<F>(&mut self, disable_interrupt: F) -> Option<(T, u8)>
    where
        F: FnOnce(),
    {
        unsafe {
            M::clear_compare();

            if let Some(instant) = self.0.peek().map(|p| p.instant) {
                let now = M::now();

                match instant.checked_duration_since(&now) {
                    None => {
                        // instant < now
                        // task became ready
                        let nr = self.0.pop_unchecked();

                        Some((nr.task, nr.index))
                    }
                    Some(_) => {
                        // TODO: Fix this hack...
                        // Extract the compare time
                        M::set_compare(*instant.duration_since_epoch().integer());

                        // Double check that the instant we set is really in the future, else
                        // dequeue. If the monotonic is fast enough it can happen that from the
                        // read of now to the set of the compare, the time can overflow. This is to
                        // guard against this.
                        if instant.checked_duration_since(&M::now()).is_none() {
                            let nr = self.0.pop_unchecked();

                            Some((nr.task, nr.index))
                        } else {
                            None
                        }

                        // Start counting down from the new reload
                        // mem::transmute::<_, SYST>(()).clear_current();
                    }
                }
            } else {
                // The queue is empty
                // mem::transmute::<_, SYST>(()).disable_interrupt();
                disable_interrupt();

                None
            }
        }
    }
}

pub struct NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
{
    pub index: u8,
    pub instant: Instant<M>,
    pub task: T,
}

impl<M, T> Eq for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
{
}

impl<M, T> Ord for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<M, T> PartialEq for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<M, T> PartialOrd for NotReady<M, T>
where
    T: Copy,
    M: Monotonic,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
