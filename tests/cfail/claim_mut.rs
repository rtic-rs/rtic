#![feature(const_fn)]

extern crate cortex_m_srp;

use cortex_m_srp::{C2, P2, Resource};

static R1: Resource<i32, C2> = Resource::new(0);

fn j1(mut prio: P2) {
    // OK only one `&mut-` reference to the data
    let r1 = R1.claim_mut(&mut prio);
}

fn j2(prio: P2) {
    // OK two `&-` references to the same data
    let r1 = R1.claim(&prio);
    let another_r1 = R1.claim(&prio);
}

fn j3(mut prio: P2) {
    // CAN'T have a `&-` reference and a `&mut-` reference to the same data
    let r1 = R1.claim(&prio);
    let another_r1 = R1.claim_mut(&mut prio);
    //~^ error
}

fn j4(mut prio: P2) {
    // CAN'T have two `&mut-` references to the same data
    let r1 = R1.claim_mut(&mut prio);
    let another_r1 = R1.claim_mut(&mut prio);
    //~^ error
}

fn main() {}
