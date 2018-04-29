use core::cmp;

use cortex_m::peripheral::{SCB, SYST};
use heapless::binary_heap::{BinaryHeap, Min};
use heapless::ArrayLength;
use typenum::{Max, Maximum, Unsigned};

use instant::Instant;
use node::{Slot, TaggedPayload};
use resource::{Resource, Threshold};

enum State<T>
where
    T: Copy,
{
    Payload(TaggedPayload<T>),
    Baseline(Instant),
    Done,
}

#[doc(hidden)]
pub struct TimerQueue<T, N>
where
    N: ArrayLength<TaggedPayload<T>>,
    T: Copy,
{
    pub syst: SYST,
    pub queue: BinaryHeap<TaggedPayload<T>, N, Min>,
}

impl<T, N> TimerQueue<T, N>
where
    N: ArrayLength<TaggedPayload<T>>,
    T: Copy,
{
    pub const fn new(syst: SYST) -> Self {
        TimerQueue {
            syst,
            queue: BinaryHeap::new(),
        }
    }

    #[inline]
    pub unsafe fn enqueue(&mut self, bl: Instant, tp: TaggedPayload<T>) {
        if self.queue
            .peek()
            .map(|head| bl < head.baseline())
            .unwrap_or(true)
        {
            self.syst.enable_interrupt();
            // set SysTick pending
            unsafe { (*SCB::ptr()).icsr.write(1 << 26) }
        }

        self.queue.push_unchecked(tp);
    }
}

pub fn dispatch<T, TQ, N, F, P>(t: &mut Threshold<P>, tq: &mut TQ, mut f: F)
where
    F: FnMut(&mut Threshold<P>, TaggedPayload<T>),
    Maximum<P, TQ::Ceiling>: Unsigned,
    N: 'static + ArrayLength<TaggedPayload<T>>,
    P: Unsigned + Max<TQ::Ceiling>,
    T: 'static + Copy + Send,
    TQ: Resource<Data = TimerQueue<T, N>>,
{
    loop {
        let state = tq.claim_mut(t, |tq, _| {
            if let Some(bl) = tq.queue.peek().map(|p| p.baseline()) {
                if Instant::now() >= bl {
                    // message ready
                    State::Payload(unsafe { tq.queue.pop_unchecked() })
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
            State::Payload(p) => f(t, p),
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
