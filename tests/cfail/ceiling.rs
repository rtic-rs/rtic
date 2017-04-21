extern crate cortex_m_rtfm as rtfm;

use rtfm::{C3, P0, P2, Resource};

static R1: Resource<(), C3> = Resource::new(());

fn j1(prio: P2) {
    let ceil = prio.as_ceiling();

    let c3 = rtfm::raise_to(ceil, &R1, |ceil| {
        // forbidden: ceiling token can't outlive critical section
        ceil  //~ error
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
