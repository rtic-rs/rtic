extern crate cortex_m_srp;

use cortex_m_srp::{C1, C2, C3, C4, C5, P2, Resource};

static R1: Resource<i32, C4> = Resource::new(0);
static R2: Resource<i32, C3> = Resource::new(0);
static R3: Resource<i32, C4> = Resource::new(0);
static R4: Resource<i32, C5> = Resource::new(0);
static R5: Resource<i32, C1> = Resource::new(0);
static R6: Resource<i32, C2> = Resource::new(0);

fn j1(prio: P2) {
    R1.lock(&prio, |r1, c3| {
        // CAN borrow a resource with ceiling C when the system ceiling SC > C
        let r2 = R2.borrow(&prio, &c3);

        // CAN borrow a resource with ceiling C when the system ceiling SC == C
        let r3 = R3.borrow(&prio, &c3);

        // CAN'T borrow a resource with ceiling C when the system ceiling SC < C
        let r4 = R4.borrow(&prio, &c3);
        //~^ error

        // CAN'T borrow a resource with ceiling C < P (task priority)
        let r5 = R5.borrow(&prio, &c3);
        //~^ error

        // CAN borrow a resource with ceiling C == P (task priority)
        let r6 = R6.borrow(&prio, &c3);
    });
}
