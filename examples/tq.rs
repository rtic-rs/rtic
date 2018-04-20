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
use cortex_m::peripheral::{DWT, ITM};
use rtfm::ll::{Consumer, FreeList, Message, Node, Payload, Producer, RingBuffer, Slot, TimerQueue};
use rtfm::{app, Resource, Threshold};
use stm32f103xx::Interrupt;

const ACAP: usize = 2;

const MS: u32 = 8_000;

app! {
    device: stm32f103xx,

    resources: {
        /* timer queue */
        static TQ: TimerQueue<Task, [Message<Task>; 2]>;

        /* a */
        // payloads w/ after
        static AN: [Node<i32>; 2] = [Node::new(), Node::new()];
        static AFL: FreeList<i32> = FreeList::new();

        static AQ: RingBuffer<(u32, i32), [(u32, i32); ACAP + 1], u8> = RingBuffer::u8();
        static AQC: Consumer<'static, (u32, i32), [(u32, i32); ACAP + 1], u8>;
        static AQP: Producer<'static, (u32, i32), [(u32, i32); ACAP + 1], u8>;

        /* exti0 */
        static Q1: RingBuffer<Task1, [Task1; ACAP + 1], u8> = RingBuffer::u8();
        static Q1C: Consumer<'static, Task1, [Task1; ACAP + 1], u8>;
        static Q1P: Producer<'static, Task1, [Task1; ACAP + 1], u8>;
    },

    init: {
        resources: [AN, Q1, AQ],
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
            resources: [AQC, Q1C],
            priority: 1,
        },

        // timer queue
        SYS_TICK: {
            path: sys_tick,
            resources: [TQ, AQP, Q1P, AFL],
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

    let (aqp, aqc) = r.AQ.split();
    let (q1p, q1c) = r.Q1.split();
    init::LateResources {
        TQ: TimerQueue::new(p.core.SYST),
        AQC: aqc,
        AQP: aqp,
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

fn a(_t: &mut Threshold, bl: u32, payload: i32) {
    let now = DWT::get_cycle_count();
    unsafe {
        iprintln!(
            &mut (*ITM::ptr()).stim[0],
            "a(now={}, bl={}, payload={})",
            now,
            bl,
            payload
        )
    }
}

fn exti1(t: &mut Threshold, r: EXTI1::Resources) {
    /* expansion */
    let bl = DWT::get_cycle_count();
    let mut async = a::Async::new(bl, r.TQ, r.AFL);
    /* end of expansion */

    unsafe { iprintln!(&mut (*ITM::ptr()).stim[0], "EXTI0(bl={})", bl) }
    async.a(t, 100 * MS, 0).unwrap();
    async.a(t, 50 * MS, 1).unwrap();
}

/* auto generated */
fn exti0(_t: &mut Threshold, mut r: EXTI0::Resources) {
    while let Some(task) = r.Q1C.dequeue() {
        match task {
            Task1::a => {
                let (bl, payload) = r.AQC.dequeue().unwrap();
                a(&mut unsafe { Threshold::new(1) }, bl, payload);
            }
        }
    }
}

fn sys_tick(t: &mut Threshold, r: SYS_TICK::Resources) {
    #[allow(non_snake_case)]
    let SYS_TICK::Resources {
        mut AFL,
        mut AQP,
        mut Q1P,
        mut TQ,
    } = r;

    enum State<T> {
        Message(Message<T>),
        Baseline(u32),
        Done,
    }

    loop {
        let state = TQ.claim_mut(t, |tq, _| {
            if let Some(m) = tq.queue.peek().cloned() {
                if (DWT::get_cycle_count() as i32).wrapping_sub(m.baseline as i32) >= 0 {
                    // message ready
                    tq.queue.pop();
                    State::Message(m)
                } else {
                    // set timeout
                    State::Baseline(m.baseline)
                }
            } else {
                // empty queue
                tq.syst.disable_interrupt();
                State::Done
            }
        });

        match state {
            State::Message(m) => {
                match m.task {
                    Task::a => {
                        // read payload
                        let (payload, slot) = unsafe { Payload::<i32>::from(m.payload) }.read();

                        // return free slot to the free list
                        AFL.claim_mut(t, |afl, _| afl.push(slot));

                        // enqueue a new `a` task
                        AQP.claim_mut(t, |aqp, t| {
                            aqp.enqueue_unchecked((m.baseline, payload));
                            Q1P.claim_mut(t, |q1p, _| {
                                q1p.enqueue_unchecked(Task1::a);
                                rtfm::set_pending(Interrupt::EXTI0);
                            });
                        });
                    }
                }
            }
            State::Baseline(bl) => {
                const MAX: u32 = 0x00ffffff;

                let diff = (bl as i32).wrapping_sub(DWT::get_cycle_count() as i32);

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

    use rtfm::ll::Message;
    use rtfm::{Resource, Threshold};
    use Task;

    #[allow(non_snake_case)]
    pub struct Async {
        // inherited baseline
        baseline: u32,
        TQ: ::EXTI1::TQ,
        AFL: ::EXTI1::AFL,
    }

    impl Async {
        #[allow(non_snake_case)]
        pub fn new(bl: u32, TQ: ::EXTI1::TQ, AFL: ::EXTI1::AFL) -> Self {
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
                        let bl = baseline.wrapping_add(after);
                        if tq.queue
                            .peek()
                            .map(|head| (bl as i32).wrapping_sub(head.baseline as i32) < 0)
                            .unwrap_or(true)
                        {
                            tq.syst.enable_interrupt();
                            // Set SYST pending
                            unsafe { (*SCB::ptr()).icsr.write(1 << 26) }
                        }

                        tq.queue
                            .push(Message::new(bl, Task::a, slot.write(payload)))
                            .ok();

                        Ok(())
                    }
                })
            } else {
                Err(payload)
            }
        }
    }
}
