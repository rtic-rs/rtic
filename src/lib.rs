//! Stack Resource Policy for Cortex-M processors
//!
//! NOTE ARMv6-M is not supported at the moment.

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

// NOTE Only the 4 highest bits of the priority byte (BASEPRI / NVIC.IPR) are
// considered when determining priorities.
const PRIORITY_BITS: u8 = 4;

extern crate cortex_m;

use cortex_m::interrupt::CsCtxt;
use cortex_m::peripheral::Peripheral;
use cortex_m::register::{basepri, basepri_max};

use core::cell::UnsafeCell;

// XXX Do we need memory / instruction / compiler barriers here?
#[inline(always)]
unsafe fn claim<T, R, F>(f: F, res: *const T, ceiling: u8) -> R
    where F: FnOnce(&T) -> R
{
    let old_basepri = basepri::read();
    basepri_max::write(ceiling);
    let ret = f(&*res);
    basepri::write(old_basepri);
    ret
}

/// A peripheral as a resource
pub struct ResourceP<T>
    where T: 'static
{
    peripheral: Peripheral<T>,
    // NOTE NVIC-style priority ceiling
    ceiling: u8,
}

impl<T> ResourceP<T> {
    /// Wraps a `peripheral` into a `Resource`
    ///
    /// NOTE `ceiling` must be in the range `[1, 15]` (inclusive)
    ///
    /// # Unsafety
    ///
    /// - Do not create two resources that point to the same peripheral
    pub const unsafe fn new(p: Peripheral<T>, ceiling: u8) -> Self {
        ResourceP {
            peripheral: p,
            // NOTE elements 1 and 2 of the tuple are a poor man's const context
            // range checker
            ceiling: (priority(ceiling), ceiling - 1, ceiling + 240).0,
        }
    }

    /// Claims the resource, blocking tasks with priority lower than `ceiling`
    pub fn claim<R, F>(&'static self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        unsafe { claim(f, self.peripheral.get(), self.ceiling) }
    }

    /// Borrows the resource for the duration of a critical section
    pub fn borrow<'a>(&'static self, _ctxt: &'a CsCtxt) -> &'a T {
        unsafe { &*self.peripheral.get() }
    }
}

unsafe impl<T> Sync for ResourceP<T> {}

/// A resource
pub struct Resource<T> {
    // NOTE NVIC-style priority ceiling
    ceiling: u8,
    data: UnsafeCell<T>,
}

impl<T> Resource<T> {
    /// Initializes a resource
    ///
    /// NOTE `ceiling` must be in the range `[1, 15]`
    pub const fn new(data: T, ceiling: u8) -> Self {
        Resource {
            // NOTE elements 1 and 2 of the tuple are a poor man's const context
            // range checker
            ceiling: (priority(ceiling), ceiling - 1, ceiling + 240).0,
            data: UnsafeCell::new(data),
        }
    }

    /// Claims the resource, blocking tasks with priority lower than `ceiling`
    pub fn claim<F, R>(&'static self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        unsafe { claim(f, self.data.get(), self.ceiling) }
    }

    /// Borrows the resource for the duration of a critical section
    pub fn borrow<'cs>(&self, _ctxt: &'cs CsCtxt) -> &'cs T {
        unsafe { &*self.data.get() }
    }
}

unsafe impl<T> Sync for Resource<T> {}

/// Turns a `logical` priority into a NVIC-style priority
///
/// With `logical` priorities, `2` has HIGHER priority than `1`.
///
/// With NVIC priorities, `32` has LOWER priority than `16`. (Also, NVIC
/// priorities encode the actual priority in the highest bits of a byte so
/// priorities like `1` and `2` aren't actually different)
///
/// NOTE `logical` must be in the range `[1, 15]` (inclusive)
pub const fn priority(logical: u8) -> u8 {
    // NOTE elements 1 and 2 of the tuple are a poor man's const context range
    // checker
    (((1 << PRIORITY_BITS) - logical) << (8 - PRIORITY_BITS),
     logical - 1,
     logical + 240)
            .0

}
