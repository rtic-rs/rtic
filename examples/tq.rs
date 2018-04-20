// #![deny(unsafe_code)]
// #![deny(warnings)]
#![allow(dead_code)]
#![feature(proc_macro)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use core::cmp;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::ITM;
use rtfm::ll::{Consumer, FreeList, Instant, Node, Producer, RingBuffer, Slot, TaggedPayload,
               TimerQueue};
use rtfm::{app, Resource, Threshold};
use stm32f103xx::Interrupt;

const ACAP: usize = 2;

const MS: u32 = 8_000;

app! {
    device: stm32f103xx,

    resources: {
        /* timer queue */
        static TQ: TimerQueue<Task, [TaggedPayload<Task>; 2]>;

        /* a */
        // payloads w/ after
        static AN: [Node<i32>; 2] = [Node::new(), Node::new()];
        static AFL: FreeList<i32> = FreeList::new();

        /* exti0 */
        static Q1: RingBuffer<TaggedPayload<Task1>, [TaggedPayload<Task1>; ACAP + 1], u8> =
            RingBuffer::u8();
        static Q1C: Consumer<'static, TaggedPayload<Task1>, [TaggedPayload<Task1>; ACAP + 1], u8>;
        static Q1P: Producer<'static, TaggedPayload<Task1>, [TaggedPayload<Task1>; ACAP + 1], u8>;
    },

    init: {
        resources: [AN, Q1],
    },

    tasks: {
        EXTI1: {
            path: exti1,
            resources: [TQ, AFL],
            priority: 1,

            // async: [a],
        },

        // dispatch interrupt
        EXTI0: {
            path: exti0,
            resources: [Q1C, AFL],
            priority: 1,
        },

        // timer queue
        SYS_TICK: {
            path: sys_tick,
            resources: [TQ, Q1P],
            priority: 2,
        },
    },
}

pub fn init(mut p: ::init::Peripherals, r: init::Resources) -> init::LateResources {
    // ..

    /* executed after `init` end */
    p.core.DWT.enable_cycle_counter();
    unsafe { p.core.DWT.cyccnt.write(0) };
    p.core.SYST.set_clock_source(SystClkSource::Core);
    p.core.SYST.enable_counter();
    p.core.SYST.disable_interrupt();

    // populate the free list
    for n in r.AN {
        r.AFL.push(Slot::new(n));
    }

    let (q1p, q1c) = r.Q1.split();
    init::LateResources {
        TQ: TimerQueue::new(p.core.SYST),
        Q1C: q1c,
        Q1P: q1p,
    }
}

pub fn idle() -> ! {
    rtfm::set_pending(Interrupt::EXTI1);

    loop {
        rtfm::wfi()
    }
}

fn a(_t: &mut Threshold, bl: Instant, payload: i32) {
    unsafe {
        iprintln!(
            &mut (*ITM::ptr()).stim[0],
            "a(now={:?}, bl={:?}, payload={})",
            Instant::now(),
            bl,
            payload
        )
    }
}

fn exti1(t: &mut Threshold, r: EXTI1::Resources) {
    /* expansion */
    let bl = Instant::now();
    let mut async = a::Async::new(bl, r.TQ, r.AFL);
    /* end of expansion */

    unsafe { iprintln!(&mut (*ITM::ptr()).stim[0], "EXTI0(bl={:?})", bl) }
    async.a(t, 100 * MS, 0).unwrap();
    async.a(t, 50 * MS, 1).unwrap();
}

/* auto generated */
fn exti0(t: &mut Threshold, mut r: EXTI0::Resources) {
    while let Some(payload) = r.Q1C.dequeue() {
        match payload.tag() {
            Task1::a => {
                let (bl, payload, slot) = unsafe { payload.coerce() }.read();

                r.AFL.claim_mut(t, |afl, _| afl.push(slot));

                a(&mut unsafe { Threshold::new(1) }, bl, payload);
            }
        }
    }
}

fn sys_tick(t: &mut Threshold, r: SYS_TICK::Resources) {
    #[allow(non_snake_case)]
    let SYS_TICK::Resources { mut Q1P, mut TQ } = r;

    enum State<T>
    where
        T: Copy,
    {
        Payload(TaggedPayload<T>),
        Baseline(Instant),
        Done,
    }

    loop {
        let state = TQ.claim_mut(t, |tq, _| {
            if let Some(bl) = tq.queue.peek().map(|p| p.baseline()) {
                if Instant::now() >= bl {
                    // message ready
                    State::Payload(tq.queue.pop().unwrap())
                } else {
                    // new timeout
                    State::Baseline(bl)
                }
            } else {
                // empty queue
                tq.syst.disable_interrupt();
                State::Done
            }
        });

        match state {
            State::Payload(p) => match p.tag() {
                Task::a => {
                    Q1P.claim_mut(t, |q1p, _| q1p.enqueue_unchecked(p.retag(Task1::a)));
                    rtfm::set_pending(Interrupt::EXTI0);
                }
            },
            State::Baseline(bl) => {
                const MAX: u32 = 0x00ffffff;

                let diff = bl - Instant::now();

                if diff < 0 {
                    // message became ready
                    continue;
                } else {
                    TQ.claim_mut(t, |tq, _| {
                        tq.syst.set_reload(cmp::min(MAX, diff as u32));
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

// Tasks dispatched at a priority of 1
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Task1 {
    a,
}

// All tasks
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Task {
    a,
}

mod a {
    use cortex_m::peripheral::SCB;

    use rtfm::ll::Instant;
    use rtfm::{Resource, Threshold};
    use Task;

    #[allow(non_snake_case)]
    pub struct Async {
        // inherited baseline
        baseline: Instant,
        TQ: ::EXTI1::TQ,
        AFL: ::EXTI1::AFL,
    }

    impl Async {
        #[allow(non_snake_case)]
        pub fn new(bl: Instant, TQ: ::EXTI1::TQ, AFL: ::EXTI1::AFL) -> Self {
            Async {
                baseline: bl,
                TQ,
                AFL,
            }
        }

        pub fn a(&mut self, t: &mut Threshold, after: u32, payload: i32) -> Result<(), i32> {
            if let Some(slot) = self.AFL.claim_mut(t, |afl, _| afl.pop()) {
                let baseline = self.baseline;
                self.TQ.claim_mut(t, |tq, _| {
                    if tq.queue.capacity() == tq.queue.len() {
                        // full
                        Err(payload)
                    } else {
                        let bl = baseline + after;
                        if tq.queue
                            .peek()
                            .map(|head| bl < head.baseline())
                            .unwrap_or(true)
                        {
                            tq.syst.enable_interrupt();
                            // Set SYST pending
                            unsafe { (*SCB::ptr()).icsr.write(1 << 26) }
                        }

                        tq.queue.push(slot.write(bl, payload).tag(Task::a)).ok();

                        Ok(())
                    }
                })
            } else {
                Err(payload)
            }
        }
    }
}
