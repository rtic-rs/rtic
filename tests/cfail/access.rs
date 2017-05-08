extern crate cortex_m_rtfm as rtfm;

use rtfm::{C1, C2, C3, C4, C5, P2, Resource, T2};

static R1: Resource<i32, C4> = Resource::new(0);
static R2: Resource<i32, C3> = Resource::new(0);
static R3: Resource<i32, C4> = Resource::new(0);
static R4: Resource<i32, C5> = Resource::new(0);
static R5: Resource<i32, C1> = Resource::new(0);
static R6: Resource<i32, C2> = Resource::new(0);

fn j1(prio: P2, thr: T2) {
    thr.raise(
        &R1, |thr| {
            // NOTE PT = Preemption Threshold, TP = Task Priority

            // CAN access a resource with ceiling RC when PT > RC
            let r2 = R2.access(&prio, thr);

            // CAN access a resource with ceiling RC when PT == RC
            let r3 = R3.access(&prio, thr);

            // CAN'T access a resource with ceiling RC when PT < RC
            let r4 = R4.access(&prio, thr);
            //~^ error

            // CAN'T access a resource with ceiling RC when RC < TP
            let r5 = R5.access(&prio, thr);
            //~^ error

            // CAN access a resource with ceiling RC when RC == tP
            let r6 = R6.access(&prio, thr);
        }
    );
}
