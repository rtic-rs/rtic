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
            // NOTE SC = System Ceiling, P = task Priority

            // CAN access a resource with ceiling RC when SC > RC
            let r2 = R2.access(&prio, ceil);

            // CAN access a resource with ceiling RC when SC == RC
            let r3 = R3.access(&prio, ceil);

            // CAN'T access a resource with ceiling RC when SC < RC
            let r4 = R4.access(&prio, ceil);
            //~^ error

            // CAN'T access a resource with ceiling RC when RC < P
            let r5 = R5.access(&prio, ceil);
            //~^ error

            // CAN access a resource with ceiling RC when RC == P
            let r6 = R6.access(&prio, ceil);
        }
    );
}
