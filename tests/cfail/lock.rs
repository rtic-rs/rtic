extern crate cortex_m_rtfm as rtfm;

use rtfm::{CMax, C2, P1, P2, P3, PMax, Resource, T1, T2, T3, TMax};

static R1: Resource<i32, C2> = Resource::new(0);

// You don't need to raise the ceiling to access a resource with ceiling equal
// to the task priority.
fn j1(prio: P2, thr: T2) {
    thr.raise(&R1, |_| {});
    //~^ error

    // OK
    let r1 = R1.access(&prio, &thr);
}

// You CAN access a resource with ceiling C from a task with priority P if C > P
// if you raise the preemption threshold first
fn j2(prio: P1, thr: T1) {
    // OK
    thr.raise(&R1, |thr| { let r1 = R1.access(&prio, thr); })
}

static R2: Resource<i32, CMax> = Resource::new(0);

// Tasks with priority less than P16 can't access a resource with ceiling CMax
fn j4(prio: P1, thr: T1) {
    thr.raise(&R2, |thr| {});
    //~^ error
}

// Only tasks with priority P16 can directly access a resource with ceiling CMax
fn j5(prio: PMax, thr: TMax) {
    // OK
    let r2 = R2.access(&prio, &thr);
}
