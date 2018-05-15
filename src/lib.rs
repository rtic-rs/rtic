//! Real Time for The Masses: high performance, predictable, bare metal task scheduler

#![allow(warnings)]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(const_fn)]
#![feature(never_type)]
#![feature(proc_macro)]
#![feature(untagged_unions)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm_macros;
extern crate heapless;
extern crate typenum;

use cortex_m::interrupt;
pub use cortex_m_rtfm_macros::app;
use typenum::consts::*;
use typenum::Unsigned;

pub use resource::{Priority, Resource};

#[doc(hidden)]
pub mod _impl;
mod resource;

/// Executes the given closure atomically
///
/// While the closure is being executed no new task can start
pub fn atomic<R, P, F>(_p: &mut Priority<P>, f: F) -> R
where
    F: FnOnce(&mut Priority<U255>) -> R,
    P: Unsigned,
{
    unsafe {
        // Sanity check
        debug_assert!(P::to_usize() <= 255);

        if P::to_u8() < 255 {
            interrupt::disable();
            let r = f(&mut Priority::_new());
            interrupt::enable();
            r
        } else {
            f(&mut Priority::_new())
        }
    }
}
