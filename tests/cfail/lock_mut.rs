extern crate cortex_m_srp;

use cortex_m_srp::{C3, C4, P2, Resource};

static R1: Resource<i32, C4> = Resource::new(0);
static R2: Resource<i32, C3> = Resource::new(0);

fn j1(mut prio: P2) {
    R1.lock_mut(
        &mut prio, |r1: &mut i32, c3| {
            let r2 = R2.borrow(&c3);
            let another_r1: &i32 = R1.borrow(&c3);
            //~^ error
        }
    );
}
