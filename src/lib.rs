//! Real Time For the Masses (RTFM) framework for ARM Cortex-M microcontrollers
//!
//! This crate is based on [the RTFM framework] created by the Embedded Systems
//! group at [Luleå University of Technology][ltu], led by Prof. Per Lindgren,
//! and uses a simplified version of the Stack Resource Policy as scheduling
//! policy (check the [references] for details).
//!
//! [the RTFM framework]: http://www.rtfm-lang.org/
//! [ltu]: https://www.ltu.se/?l=en
//! [per]: https://www.ltu.se/staff/p/pln-1.11258?l=en
//! [references]: ./index.html#references
//!
//! # Features
//!
//! - **Event triggered tasks** as the unit of concurrency.
//! - Support for prioritization of tasks and, thus, **preemptive
//!   multitasking**.
//! - **Efficient and data race free memory sharing** through fine grained *non
//!   global* critical sections.
//! - **Deadlock free execution** guaranteed at compile time.
//! - **Minimal scheduling overhead** as the scheduler has no "software
//!   component": the hardware does all the scheduling.
//! - **Highly efficient memory usage**: All the tasks share a single call stack
//!   and there's no hard dependency on a dynamic memory allocator.
//! - **All Cortex M devices are fully supported**.
//! - This task model is amenable to known WCET (Worst Case Execution Time)
//!   analysis and scheduling analysis techniques. (Though we haven't yet
//!   developed Rust friendly tooling for that.)
//!
//! # Constraints
//!
//! - Tasks must run to completion. That's it, tasks can't contain endless
//!   loops. However, you can run an endless event loop in the `idle` *loop*.
//!
//! - Task priorities must remain constant at runtime.
//!
//! # Dependencies
//!
//! The application crate must depend on a device crate generated using
//! [`svd2rust`] v0.12.x and the "rt" feature of that crate must be enabled. The
//! SVD file used to generate the device crate *must* contain [`<cpu>`]
//! information.
//!
//! [`svd2rust`]: https://docs.rs/svd2rust/0.12.0/svd2rust/
//! [`<cpu>`]: https://www.keil.com/pack/doc/CMSIS/SVD/html/elem_cpu.html
//!
//! # `app!`
//!
//! The `app!` macro is documented [here].
//!
//! [here]: https://docs.rs/cortex-m-rtfm-macros/0.3.0/cortex_m_rtfm_macros/fn.app.html
//!
//! # Important: Cortex-M7 devices
//!
//! If targeting a Cortex-M7 device with revision r0p1 then you MUST enable the `cm7-r0p1` Cargo
//! feature of this crate or the `Resource.claim` and `Resource.claim_mut` methods WILL misbehave.
//!
//! # Examples
//!
//! In increasing grade of complexity. See the [examples](./examples/index.html)
//! module.
//!
//! # References
//!
//! - Baker, T. P. (1991). Stack-based scheduling of realtime processes.
//!   *Real-Time Systems*, 3(1), 67-99.
//!
//! > The original Stack Resource Policy paper. [PDF][srp].
//!
//! [srp]: http://www.cs.fsu.edu/~baker/papers/mstacks3.pdf
//!
//! - Eriksson, J., Häggström, F., Aittamaa, S., Kruglyak, A., & Lindgren, P.
//!   (2013, June). Real-time for the masses, step 1: Programming API and static
//!   priority SRP kernel primitives. In Industrial Embedded Systems (SIES),
//!   2013 8th IEEE International Symposium on (pp. 110-113). IEEE.
//!
//! > A description of the RTFM task and resource model. [PDF][rtfm]
//!
//! [rtfm]: http://www.diva-portal.org/smash/get/diva2:1005680/FULLTEXT01.pdf
#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_rtfm_macros;
extern crate rtfm_core;
extern crate untagged_option;

use core::{mem, u8};

pub use cortex_m::asm::{bkpt, wfi};
pub use cortex_m_rt::entry;
pub use cortex_m_rtfm_macros::app;
pub use rtfm_core::{Resource, Threshold};
#[doc(hidden)]
pub use untagged_option::UntaggedOption;

use cortex_m::interrupt::{self, Nr};
use cortex_m::peripheral::NVIC;
#[cfg(not(armv6m))]
use cortex_m::register::basepri;

pub mod examples;

/// Executes the closure `f` in a preemption free context
///
/// During the execution of the closure no task can preempt the current task.
pub fn atomic<R, F>(t: &mut Threshold, f: F) -> R
where
    F: FnOnce(&mut Threshold) -> R,
{
    if t.value() == u8::MAX {
        f(t)
    } else {
        interrupt::disable();
        let r = f(&mut unsafe { Threshold::max() });
        unsafe { interrupt::enable() };
        r
    }
}

#[inline]
#[doc(hidden)]
pub unsafe fn claim<T, R, F>(
    data: T,
    ceiling: u8,
    _nvic_prio_bits: u8,
    t: &mut Threshold,
    f: F,
) -> R
where
    F: FnOnce(T, &mut Threshold) -> R,
{
    if ceiling > t.value() {
        match () {
            #[cfg(armv6m)]
            () => atomic(t, |t| f(data, t)),

            #[cfg(not(armv6m))]
            () => {
                let max_priority = 1 << _nvic_prio_bits;

                if ceiling == max_priority {
                    atomic(t, |t| f(data, t))
                } else {
                    let old = basepri::read();
                    let hw = (max_priority - ceiling) << (8 - _nvic_prio_bits);
                    basepri::write(hw);
                    let ret = f(data, &mut Threshold::new(ceiling));
                    basepri::write(old);
                    ret
                }
            }
        }
    } else {
        f(data, t)
    }
}

/// Sets an interrupt, that is a task, as pending
///
/// If the task priority is high enough the task will be serviced immediately,
/// otherwise it will be serviced at some point after the current task ends.
pub fn set_pending<I>(interrupt: I)
where
    I: Nr,
{
    // NOTE(safe) atomic write
    let mut nvic: NVIC = unsafe { mem::transmute(()) };
    nvic.set_pending(interrupt);
}
