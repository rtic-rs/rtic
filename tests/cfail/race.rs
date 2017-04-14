extern crate cortex_m_srp as srp;

use srp::{C2, C4, P1, P3, Resource};

static R1: Resource<i32, C2> = Resource::new(0);
static R2: Resource<i32, C4> = Resource::new(0);

fn j1(prio: P1) {
    R1.lock(&prio, |r1, _| {
        // Would preempt this critical section
        // srp::request(j2);
    });
}

fn j2(prio: P3) {
    R2.lock(&prio, |r2, c4| {
        // OK  C2 (R1's ceiling) <= C4 (system ceiling)
        // BAD C2 (R1's ceiling) <  P3 (j2's priority)
        let r1 = R1.borrow(&prio, &c4);
        //~^ error
    });
}
