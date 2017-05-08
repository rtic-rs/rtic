extern crate cortex_m_rtfm as rtfm;

use rtfm::{C2, C4, P1, P3, Resource, T1, T3};

static R1: Resource<i32, C2> = Resource::new(0);
static R2: Resource<i32, C4> = Resource::new(0);

fn j1(prio: P1, thr: T1) {
    thr.raise(
        &R1, |thr| {
            let r1 = R1.access(&prio, thr);

            // `j2` preempts this critical section
            rtfm::request(j2);
        }
    );
}

fn j2(_task: Task, prio: P3, thr: T3) {
    thr.raise(
        &R2, |thr| {
            // OK  C2 (R1's ceiling) <= T4 (preemption threshold)
            // BAD C2 (R1's ceiling) <  P3 (j2's priority)
            let r1 = R1.access(&prio, thr);
            //~^ error
        }
    );
}

// glue
extern crate cortex_m;

use cortex_m::ctxt::Context;
use cortex_m::interrupt::Nr;

struct Task;

unsafe impl Context for Task {}
unsafe impl Nr for Task {
    fn nr(&self) -> u8 {
        0
    }
}
