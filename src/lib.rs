//! Real Time For the Masses (RTFM), a framework for building concurrent
//! applications, for ARM Cortex-M microcontrollers
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
//!   loops. However, you can run an endless event loop in the `idle` function.
//!
//! - Task priorities must remain constant at runtime.
//!
//! # Dependencies
//!
//! - A device crate generated using [`svd2rust`] v0.11.x. The input SVD file
//!   *must* contain [`<cpu>`] information.
//! - A `start` lang time: Vanilla `main` must be supported in binary crates.
//!   You can use the [`cortex-m-rt`] crate to fulfill the requirement
//!
//! [`svd2rust`]: https://docs.rs/svd2rust/0..0/svd2rust/
//! [`<cpu>`]: https://www.keil.com/pack/doc/CMSIS/SVD/html/elem_cpu.html
//! [`cortex-m-rt`]: https://docs.rs/cortex-m-rt/0.3.0/cortex_m_rt/
//!
//! # Examples
//!
//! In increasing grade of complexity, see the [examples](./examples/index.html)
//! module.
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(optin_builtin_traits)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm_macros;
extern crate rtfm_core;
extern crate static_ref;

use core::u8;

pub use rtfm_core::{Resource, Static, Threshold};
pub use cortex_m::asm::{bkpt, wfi};
pub use cortex_m_rtfm_macros::app;
use cortex_m::interrupt::{self, Nr};
#[cfg(not(armv6m))]
use cortex_m::register::basepri;

pub mod examples;

/// Executes the closure `f` in an interrupt free context
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

/// Sets an interrupt as pending
///
/// If the interrupt priority is high enough the interrupt will be serviced
/// immediately, otherwise it will be serviced at some point after the current
/// task ends.
pub fn set_pending<I>(interrupt: I)
where
    I: Nr,
{
    // NOTE(safe) atomic write
    let nvic = unsafe { &*cortex_m::peripheral::NVIC.get() };
    nvic.set_pending(interrupt);
}
