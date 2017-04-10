extern crate cortex_m_srp;

use cortex_m_srp::{C2, C3, C4, P1, Resource};

static R1: Resource<i32, C3> = Resource::new(0);
static R2: Resource<i32, C2> = Resource::new(0);
static R3: Resource<i32, C3> = Resource::new(0);
static R4: Resource<i32, C4> = Resource::new(0);

fn j1(prio: P1) {
    R1.lock(&prio, |r1, c3| {
        // CAN borrow a resource with ceiling C when the system ceiling SC > C
        let r2 = R2.borrow(&c3);

        // CAN borrow a resource with ceiling C when the system ceiling SC == C
        let r3 = R3.borrow(&c3);

        // CAN'T borrow a resource with ceiling C when the system ceiling SC < C
        let r4 = R4.borrow(&c3);
        //~^ error
    });
}
