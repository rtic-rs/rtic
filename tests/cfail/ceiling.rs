extern crate cortex_m_srp;

use cortex_m_srp::{C3, P2, Resource};

static R1: Resource<(), C3> = Resource::new(());

fn j1(prio: P2) {
    let c3 = R1.lock(&prio, |r1, c3| {
        // forbidden: ceiling token can't outlive critical section
        c3  //~ error
    });

    // Would be bad: lockless access to a resource with ceiling = 3
    let r2 = R1.borrow(&prio, c3);
}
