extern crate cortex_m_rtfm as rtfm;

use rtfm::{C1, C2, C3, C4, C5, P2, Resource};

static R1: Resource<i32, C4> = Resource::new(0);
static R2: Resource<i32, C3> = Resource::new(0);
static R3: Resource<i32, C4> = Resource::new(0);
static R4: Resource<i32, C5> = Resource::new(0);
static R5: Resource<i32, C1> = Resource::new(0);
static R6: Resource<i32, C2> = Resource::new(0);

fn j1(prio: P2) {
    let ceil = prio.as_ceiling();

    ceil.raise(
        &R1, |ceil| {
            // NOTE CC = Current Ceiling, P = task Priority

            // CAN borrow a resource with ceiling RC when CC > RC
            let r2 = R2.borrow(&prio, ceil);

            // CAN borrow a resource with ceiling RC when CC == RC
            let r3 = R3.borrow(&prio, ceil);

            // CAN'T borrow a resource with ceiling RC when CC < RC
            let r4 = R4.borrow(&prio, ceil);
            //~^ error

            // CAN'T borrow a resource with ceiling RC when RC < P
            let r5 = R5.borrow(&prio, ceil);
            //~^ error

            // CAN borrow a resource with ceiling RC when RC == P
            let r6 = R6.borrow(&prio, ceil);
        }
    );
}
