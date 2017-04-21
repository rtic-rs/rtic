extern crate cortex_m_rtfm as rtfm;

use rtfm::{C16, C2, P1, P16, P2, P3, Resource};

static R1: Resource<i32, C2> = Resource::new(0);

// You CAN'T lock a resource with ceiling C from a task with priority P if P > C
fn j1(prio: P3) {
    let ceil = prio.as_ceiling();

    ceil.raise(&R1, |ceil| {
        //~^ error
    });
}

// DON'T lock a resource with ceiling equal to the task priority.
// Instead use `borrow`
fn j2(prio: P2) {
    let ceil = prio.as_ceiling();

    ceil.raise(&R1, |_| {});
    //~^ error

    // OK
    let r1 = R1.borrow(&prio, ceil);
}

// You CAN lock a resource with ceiling C from a task with priority P if C > P
fn j3(prio: P1) {
    let ceil = prio.as_ceiling();

    // OK
    ceil.raise(&R1, |ceil| {
        let r1 = R1.borrow(&prio, ceil);
    })
}

static R2: Resource<i32, C16> = Resource::new(0);

// Tasks with priority less than P16 can't lock a resource with ceiling C16
fn j4(prio: P1) {
    let ceil = prio.as_ceiling();

    ceil.raise(&R2, |ceil| {
        //~^ error
    });
}

// Only tasks with priority P16 can claim a resource with ceiling C16
fn j5(prio: P16) {
    // OK
    let r2 = R2.borrow(&prio, prio.as_ceiling());
}
