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

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::{DWT, ITM, SCB};
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
        static AN0: Node<u32> = Node::new();
        static AN1: Node<u32> = Node::new();
        static AFL: FreeList<u32> = FreeList::new();

        // payloads w/o after
        static AQ: RingBuffer<u32, [u32; ACAP + 1]> = RingBuffer::new();
        static AQC: Consumer<'static, u32, [u32; ACAP + 1]>;
        static AQP: Producer<'static, u32, [u32; ACAP + 1]>;

        /* exti0 */
        static Q1: RingBuffer<Task1, [Task1; ACAP + 1]> = RingBuffer::new();
        static Q1C: Consumer<'static, Task1, [Task1; ACAP + 1]>;
        static Q1P: Producer<'static, Task1, [Task1; ACAP + 1]>;
    },

    init: {
        resources: [AN0, AN1, Q1, AQ],
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
            priority: 1,
        },
    },
}

pub fn init(mut p: ::init::Peripherals, r: init::Resources) -> init::LateResources {
    // ..

    /* executed after `init` end */
    p.core.DWT.enable_cycle_counter();
    unsafe { p.core.DWT.cyccnt.write(0) };
    p.core.SYST.set_clock_source(SystClkSource::Core);
    p.core.SYST.enable_interrupt();

    // populate the free list
    r.AFL.push(Slot::new(r.AN0));
    r.AFL.push(Slot::new(r.AN1));

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

fn a(_t: &mut Threshold, payload: u32) {
    let bl = DWT::get_cycle_count();
    unsafe {
        iprintln!(
            &mut (*ITM::ptr()).stim[0],
            "a(bl={}, payload={})",
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
    // rtfm::bkpt();
}

/* auto generated */
fn exti0(_t: &mut Threshold, mut r: EXTI0::Resources) {
    while let Some(task) = r.Q1C.dequeue() {
        match task {
            Task1::a => {
                let payload = r.AQC.dequeue().unwrap_or_else(|| unreachable!());
                a(&mut unsafe { Threshold::new(1) }, payload);
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

    TQ.claim_mut(t, |tq, t| {
        tq.syst.disable_counter();

        if let Some(m) = tq.queue.pop() {
            match m.task {
                Task::a => {
                    // read payload
                    let (payload, slot) = unsafe { Payload::<u32>::from(m.payload) }.read();

                    // enqueue a new `a` task
                    AQP.claim_mut(t, |aqp, t| {
                        aqp.enqueue(payload).ok().unwrap();
                        Q1P.claim_mut(t, |q1p, _| {
                            q1p.enqueue(Task1::a).ok().unwrap_or_else(|| unreachable!());
                            rtfm::set_pending(Interrupt::EXTI0);
                        });
                    });

                    // return free slot to the free list
                    AFL.claim_mut(t, |afl, _| afl.push(slot));
                }
            }

            if let Some(m) = tq.queue.peek().cloned() {
                // set up a new interrupt
                let now = DWT::get_cycle_count();

                if let Some(timeout) = tq.baseline.wrapping_add(m.deadline).checked_sub(now) {
                    // TODO deal with the 24-bit limit
                    tq.syst.set_reload(timeout);
                    tq.syst.clear_current();
                    tq.syst.enable_counter();

                    // update the timer queue baseline
                    tq.baseline = now;
                    tq.queue.iter_mut().for_each(|m| m.deadline -= timeout);
                } else {
                    // next message already expired, pend immediately
                    // NOTE(unsafe) atomic write to a stateless (from the programmer PoV) register
                    unsafe { (*SCB::ptr()).icsr.write(1 << 26) }
                }
            } else {
                // no message left to process
            }
        } else {
            unreachable!()
        }
    });
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
    use rtfm::{Resource, Threshold};
    use Task;

    #[allow(non_snake_case)]
    pub struct Async {
        bl: u32,
        TQ: ::EXTI1::TQ,
        AFL: ::EXTI1::AFL,
    }

    impl Async {
        #[allow(non_snake_case)]
        pub fn new(bl: u32, TQ: ::EXTI1::TQ, AFL: ::EXTI1::AFL) -> Self {
            Async { bl, TQ, AFL }
        }

        pub fn a(&mut self, t: &mut Threshold, after: u32, payload: u32) -> Result<(), u32> {
            if let Some(slot) = self.AFL.claim_mut(t, |afl, _| afl.pop()) {
                let bl = self.bl;
                self.TQ
                    .claim_mut(t, |tq, _| tq.insert(bl, after, Task::a, payload, slot))
                    .map_err(|(p, slot)| {
                        self.AFL.claim_mut(t, |afl, _| afl.push(slot));
                        p
                    })
            } else {
                Err(payload)
            }
        }
    }
}
