extern crate cortex_m_rtfm as rtfm;

use rtfm::{C2, C3, P0, P2, Resource, T2};

static R1: Resource<(), C3> = Resource::new(());

fn j1(prio: P2, thr: T2) {
    let t3 = thr.raise(
        &R1, |thr| {
            // forbidden: ceiling token can't outlive the critical section
            thr //~ error
        }
    );

    // Would be bad: lockless access to a resource with ceiling = 3
    let r2 = R1.access(&prio, t3);
}

fn j2(prio: P0) {
    let c16 = rtfm::atomic(
        |c16| {
            // forbidden: ceiling token can't outlive the critical section
            c16 //~ error
        },
    );

    // Would be bad: lockless access to a resource with ceiling = 16
    let r1 = R1.access(&prio, c16);
}
