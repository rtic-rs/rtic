// #![deny(missing_docs)]
// #![deny(warnings)]
#![allow(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![feature(untagged_unions)]
#![feature(never_type)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm_macros;
extern crate heapless;
extern crate typenum;

use core::mem;

use cortex_m::interrupt::{self, Nr};
pub use cortex_m_rtfm_macros::app;
use heapless::ring_buffer::RingBuffer;
use typenum::consts::*;
use typenum::Unsigned;

pub use resource::{Priority, Resource};

#[doc(hidden)]
pub mod _impl;
mod resource;

/// TODO
pub fn atomic<R, P, F>(t: &mut Priority<P>, f: F) -> R
where
    F: FnOnce(&mut Priority<U255>) -> R,
    P: Unsigned,
{
    unsafe {
        // Sanity check
        debug_assert!(P::to_u8() <= 255);

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
