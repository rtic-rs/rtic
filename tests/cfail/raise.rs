extern crate cortex_m_rtfm as rtfm;

use rtfm::{C2, CMax, P1, P3, Resource, T1, T3};

static R1: Resource<i32, C2> = Resource::new(0);

// You CAN'T use `raise` to lower the preemption level
fn j1(prio: P3, thr: T3) {
    thr.raise(&R1, |thr| {});
    //~^ error
}

static R2: Resource<i32, CMax> = Resource::new(0);

// You CAN'T `raise` the preemption level to the maximum
fn j2(prio: P1, thr: T1) {
    thr.raise(&R2, |thr| {});
    //~^ error

    // Instead use `rtfm::atomic` to access a resource with ceiling C16
    rtfm::atomic(|thr| {
        let r2 = R2.access(&prio, thr);
    });
}
