extern crate cortex_m_rtfm as rtfm;

use rtfm::{C16, C1, C2, C3, P1, P16, P2, P3, Resource};

static R1: Resource<i32, C2> = Resource::new(0);

// You CAN'T use `raise` to lower the system ceiling
fn j1(prio: P3, ceil: C3) {
    ceil.raise(&R1, |ceil| {});
    //~^ error
}

// You don't need to raise the ceiling to access a resource with ceiling equal
// to the task priority.
fn j2(prio: P2, ceil: C2) {
    ceil.raise(&R1, |_| {});
    //~^ error

    // OK
    let r1 = R1.access(&prio, &ceil);
}

// You CAN access a resource with ceiling C from a task with priority P if C > P
// and you raise the ceiling first
fn j3(prio: P1, ceil: C1) {
    // OK
    ceil.raise(&R1, |ceil| { let r1 = R1.access(&prio, ceil); })
}

static R2: Resource<i32, C16> = Resource::new(0);

// Tasks with priority less than P16 can't access a resource with ceiling C16
fn j4(prio: P1, ceil: C1) {
    ceil.raise(&R2, |ceil| {});
    //~^ error
}

// Only tasks with priority P16 can access a resource with ceiling C16
fn j5(prio: P16, ceil: C16) {
    // OK
    let r2 = R2.access(&prio, &ceil);
}
