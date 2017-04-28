extern crate cortex_m_rtfm as rtfm;

use rtfm::{C1, C2, C3, C4, P1, P3, Resource};

static R1: Resource<i32, C2> = Resource::new(0);
static R2: Resource<i32, C4> = Resource::new(0);

fn j1(prio: P1, ceil: C1) {
    ceil.raise(
        &R1, |ceil| {
            let r1 = R1.access(&prio, ceil);

            // `j2` preempts this critical section
            rtfm::request(j2);
        }
    );
}

fn j2(_task: Task, prio: P3, ceil: C3) {
    ceil.raise(
        &R2, |ceil| {
            // OK  C2 (R1's ceiling) <= C4 (system ceiling)
            // BAD C2 (R1's ceiling) <  P3 (j2's priority)
            let r1 = R1.access(&prio, ceil);
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
