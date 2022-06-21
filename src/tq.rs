use crate::{
    sll::{IntrusiveSortedLinkedList, Min as IsslMin, Node as IntrusiveNode},
    Monotonic,
};
use core::cmp::Ordering;
use core::task::Waker;
use heapless::sorted_linked_list::{LinkedIndexU16, Min as SllMin, SortedLinkedList};

pub struct TimerQueue<'a, Mono, Task, const N_TASK: usize>
where
    Mono: Monotonic,
    Task: Copy,
{
    pub task_queue: SortedLinkedList<TaskNotReady<Mono, Task>, LinkedIndexU16, SllMin, N_TASK>,
    pub waker_queue: IntrusiveSortedLinkedList<'a, WakerNotReady<Mono>, IsslMin>,
}

impl<'a, Mono, Task, const N_TASK: usize> TimerQueue<'a, Mono, Task, N_TASK>
where
    Mono: Monotonic + 'a,
    Task: Copy,
{
    fn check_if_enable<F1, F2>(
        &self,
        instant: Mono::Instant,
        enable_interrupt: F1,
        pend_handler: F2,
        mono: Option<&mut Mono>,
    ) where
        F1: FnOnce(),
        F2: FnOnce(),
    {
        // Check if the top contains a non-empty element and if that element is
        // greater than nr
        let if_task_heap_max_greater_than_nr = self
            .task_queue
            .peek()
            .map_or(true, |head| instant < head.instant);
        let if_waker_heap_max_greater_than_nr = self
            .waker_queue
            .peek()
            .map_or(true, |head| instant < head.instant);

        if if_task_heap_max_greater_than_nr || if_waker_heap_max_greater_than_nr {
            if Mono::DISABLE_INTERRUPT_ON_EMPTY_QUEUE && self.is_empty() {
                if let Some(mono) = mono {
                    mono.enable_timer();
                }
                enable_interrupt();
            }

            pend_handler();
        }
    }

    /// Enqueue a task without checking if it is full
    #[inline]
    pub unsafe fn enqueue_task_unchecked<F1, F2>(
        &mut self,
        nr: TaskNotReady<Mono, Task>,
        enable_interrupt: F1,
        pend_handler: F2,
        mono: Option<&mut Mono>,
    ) where
        F1: FnOnce(),
        F2: FnOnce(),
    {
        self.check_if_enable(nr.instant, enable_interrupt, pend_handler, mono);
        self.task_queue.push_unchecked(nr);
    }

    /// Enqueue a waker
    #[inline]
    pub fn enqueue_waker<F1, F2>(
        &mut self,
        nr: &'a mut IntrusiveNode<WakerNotReady<Mono>>,
        enable_interrupt: F1,
        pend_handler: F2,
        mono: Option<&mut Mono>,
    ) where
        F1: FnOnce(),
        F2: FnOnce(),
    {
        self.check_if_enable(nr.val.instant, enable_interrupt, pend_handler, mono);
        self.waker_queue.push(nr);
    }

    /// Check if all the timer queue is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.task_queue.is_empty() && self.waker_queue.is_empty()
    }

    /// Cancel the marker value for a task
    pub fn cancel_task_marker(&mut self, marker: u32) -> Option<(Task, u8)> {
        if let Some(val) = self.task_queue.find_mut(|nr| nr.marker == marker) {
            let nr = val.pop();

            Some((nr.task, nr.index))
        } else {
            None
        }
    }

    /// Cancel the marker value for a waker
    pub fn cancel_waker_marker(&mut self, marker: u32) {
        if let Some(val) = self.waker_queue.find_mut(|nr| nr.marker == marker) {
            let _ = val.pop();
        }
    }

    /// Update the instant at an marker value for a task to a new instant
    #[allow(clippy::result_unit_err)]
    pub fn update_task_marker<F: FnOnce()>(
        &mut self,
        marker: u32,
        new_marker: u32,
        instant: Mono::Instant,
        pend_handler: F,
    ) -> Result<(), ()> {
        if let Some(mut val) = self.task_queue.find_mut(|nr| nr.marker == marker) {
            val.instant = instant;
            val.marker = new_marker;

            // On update pend the handler to reconfigure the next compare match
            pend_handler();

            Ok(())
        } else {
            Err(())
        }
    }

    fn dequeue_task_queue(
        &mut self,
        instant: Mono::Instant,
        mono: &mut Mono,
    ) -> Option<(Task, u8)> {
        let now = mono.now();
        if instant <= now {
            // task became ready
            let nr = unsafe { self.task_queue.pop_unchecked() };
            Some((nr.task, nr.index))
        } else {
            // Set compare
            mono.set_compare(instant);

            // Double check that the instant we set is really in the future, else
            // dequeue. If the monotonic is fast enough it can happen that from the
            // read of now to the set of the compare, the time can overflow. This is to
            // guard against this.
            if instant <= now {
                let nr = unsafe { self.task_queue.pop_unchecked() };
                Some((nr.task, nr.index))
            } else {
                None
            }
        }
    }

    fn dequeue_waker_queue(&mut self, instant: Mono::Instant, mono: &mut Mono) {
        let now = mono.now();
        if instant <= now {
            // Task became ready, wake the waker
            if let Some(v) = self.waker_queue.pop() {
                v.val.waker.wake_by_ref()
            }
        } else {
            // Set compare
            mono.set_compare(instant);

            // Double check that the instant we set is really in the future, else
            // dequeue. If the monotonic is fast enough it can happen that from the
            // read of now to the set of the compare, the time can overflow. This is to
            // guard against this.
            if instant <= now {
                if let Some(v) = self.waker_queue.pop() {
                    v.val.waker.wake_by_ref()
                }
            }
        }
    }

    /// Dequeue a task from the ``TimerQueue``
    pub fn dequeue<F>(&mut self, disable_interrupt: F, mono: &mut Mono) -> Option<(Task, u8)>
    where
        F: FnOnce(),
    {
        mono.clear_compare_flag();

        let tq = self.task_queue.peek().map(|p| p.instant);
        let wq = self.waker_queue.peek().map(|p| p.instant);

        let dequeue_task;
        let instant;

        match (tq, wq) {
            (Some(tq_instant), Some(wq_instant)) => {
                if tq_instant <= wq_instant {
                    dequeue_task = true;
                    instant = tq_instant;
                } else {
                    dequeue_task = false;
                    instant = wq_instant;
                }
            }
            (Some(tq_instant), None) => {
                dequeue_task = true;
                instant = tq_instant;
            }
            (None, Some(wq_instant)) => {
                dequeue_task = false;
                instant = wq_instant;
            }
            (None, None) => {
                // The queue is empty, disable the interrupt.
                if Mono::DISABLE_INTERRUPT_ON_EMPTY_QUEUE {
                    disable_interrupt();
                    mono.disable_timer();
                }

                return None;
            }
        }

        if dequeue_task {
            self.dequeue_task_queue(instant, mono)
        } else {
            self.dequeue_waker_queue(instant, mono);
            None
        }
    }
}

pub struct TaskNotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    pub task: Task,
    pub index: u8,
    pub instant: Mono::Instant,
    pub marker: u32,
}

impl<Mono, Task> Eq for TaskNotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
}

impl<Mono, Task> Ord for TaskNotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<Mono, Task> PartialEq for TaskNotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<Mono, Task> PartialOrd for TaskNotReady<Mono, Task>
where
    Task: Copy,
    Mono: Monotonic,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    pub waker: Waker,
    pub instant: Mono::Instant,
    pub marker: u32,
}

impl<Mono> Eq for WakerNotReady<Mono> where Mono: Monotonic {}

impl<Mono> Ord for WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<Mono> PartialEq for WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<Mono> PartialOrd for WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
