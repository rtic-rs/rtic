extern crate cortex_m_srp as rtfm;

use rtfm::{C3, P0, P2, Resource};

static R1: Resource<(), C3> = Resource::new(());

fn j1(prio: P2) {
    let c3 = R1.lock(&prio, |r1, c3| {
        // forbidden: ceiling token can't outlive critical section
        c3  //~ error
    });

    // Would be bad: lockless access to a resource with ceiling = 3
    let r2 = R1.borrow(&prio, c3);
}

fn j2(prio: P0) {
    let c16 = rtfm::critical(|c16| {
        // forbidden: ceiling token can't outlive critical section
        c16  //~ error
    });

    // Would be bad: lockless access to a resource with ceiling = 16
    let r1 = R1.borrow(&prio, c16);
}
