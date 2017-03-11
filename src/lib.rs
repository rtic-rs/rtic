//! Stack Resource Policy for Cortex-M processors
//!
//! NOTE ARMv6-M is not supported at the moment.

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;

use cortex_m::ctxt::Context;
use cortex_m::peripheral::Peripheral;
use cortex_m::register::{basepri, basepri_max};

use core::cell::UnsafeCell;
use core::marker::PhantomData;

// NOTE Only the 4 highest bits of the priority byte (BASEPRI / NVIC.IPR) are
// considered when determining priorities.
const PRIORITY_BITS: u8 = 4;

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
pub struct ResourceP<P, Ceiling>
    where P: 'static
{
    _marker: PhantomData<Ceiling>,
    peripheral: Peripheral<P>,
}

impl<P, C> ResourceP<P, C>
    where C: CeilingLike
{
    /// Wraps a `peripheral` into a `Resource`
    ///
    /// # Unsafety
    ///
    /// - Must not create two resources that point to the same peripheral
    /// - The ceiling, `C`, must be picked to prevent two or more tasks from
    ///   concurrently accessing the resource through preemption
    pub const unsafe fn new(p: Peripheral<P>) -> Self {
        ResourceP {
            _marker: PhantomData,
            peripheral: p,
        }
    }
}

impl<P, C> ResourceP<P, C>
    where C: Ceiling
{
    /// Borrows the resource without locking
    // TODO document unsafety
    pub unsafe fn borrow<'ctxt, Ctxt>(&'static self,
                                      _ctxt: &'ctxt Ctxt)
                                      -> &'ctxt P
        where Ctxt: Context
    {
        &*self.peripheral.get()
    }

    /// Locks the resource, blocking tasks with priority lower than `Ceiling`
    pub fn lock<R, F>(&'static self, f: F) -> R
        where F: FnOnce(&P) -> R,
              C: Ceiling
    {
        unsafe { claim(f, self.peripheral.get(), C::ceiling()) }
    }
}

impl<P> ResourceP<P, C0> {
    /// Borrows the resource without locking
    pub fn borrow<'ctxt, Ctxt>(&'static self, _ctxt: &'ctxt Ctxt) -> &'ctxt P
        where Ctxt: Context
    {
        unsafe { &*self.peripheral.get() }
    }
}

unsafe impl<P, C> Sync for ResourceP<P, C> {}

/// A resource
pub struct Resource<T, Ceiling> {
    _marker: PhantomData<Ceiling>,
    data: UnsafeCell<T>,
}

impl<T, C> Resource<T, C> {
    /// Initializes a resource
    ///
    /// # Unsafety
    ///
    /// - The ceiling, `C`, must be picked to prevent two or more tasks from
    ///   concurrently accessing the resource through preemption
    pub const unsafe fn new(data: T) -> Self
        where C: CeilingLike
    {
        Resource {
            _marker: PhantomData,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T, C> Resource<T, C>
    where C: Ceiling
{
    /// Locks the resource, blocking tasks with priority lower than `ceiling`
    pub fn lock<F, R>(&'static self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        unsafe { claim(f, self.data.get(), C::ceiling()) }
    }

    /// Borrows the resource, without locking
    pub unsafe fn borrow<'ctxt, Ctxt>(&'static self,
                                      _ctxt: &'ctxt Ctxt)
                                      -> &'ctxt T
        where Ctxt: Context
    {
        &*self.data.get()
    }
}

impl<T> Resource<T, C0> {
    /// Borrows the resource without locking
    pub fn borrow<'ctxt, Ctxt>(&'static self, _ctxt: &'ctxt Ctxt) -> &'ctxt T
        where Ctxt: Context
    {
        unsafe { &*self.data.get() }
    }
}

unsafe impl<T, Ceiling> Sync for Resource<T, Ceiling> {}

/// Turns a `logical` priority into a NVIC-style priority
///
/// With `logical` priorities, `2` has HIGHER priority than `1`.
///
/// With NVIC priorities, `32` has LOWER priority than `16`. (Also, NVIC
/// priorities encode the actual priority in the highest bits of a byte so
/// priorities like `1` and `2` aren't actually different)
///
/// NOTE `logical` must be in the range `[1, 15]` (inclusive)
pub const fn logical(priority: u8) -> u8 {
    // NOTE elements 1 and 2 of the tuple are a poor man's const context range
    // checker
    (((1 << PRIORITY_BITS) - priority) << (8 - PRIORITY_BITS),
     priority - 1,
     priority + 240)
            .0

}

/// Fake ceiling, indicates that the resource is shared by cooperative tasks
pub struct C0 {
    _0: (),
}

/// Ceiling
pub struct C1 {
    _0: (),
}

/// Ceiling
pub struct C2 {
    _0: (),
}

/// Ceiling
pub struct C3 {
    _0: (),
}

/// Ceiling
pub struct C4 {
    _0: (),
}

/// Ceiling
pub struct C5 {
    _0: (),
}

/// Ceiling
pub struct C6 {
    _0: (),
}

/// Ceiling
pub struct C7 {
    _0: (),
}

/// Ceiling
pub struct C8 {
    _0: (),
}

/// Ceiling
pub struct C9 {
    _0: (),
}

/// Ceiling
pub struct C10 {
    _0: (),
}

/// Ceiling
pub struct C11 {
    _0: (),
}

/// Ceiling
pub struct C12 {
    _0: (),
}

/// Ceiling
pub struct C13 {
    _0: (),
}

/// Ceiling
pub struct C14 {
    _0: (),
}

/// Ceiling
pub struct C15 {
    _0: (),
}

/// A real ceiling
// XXX this should be a "closed" trait
pub unsafe trait Ceiling {
    /// Returns the ceiling as a number
    fn ceiling() -> u8;
}

/// Usable as a ceiling
pub unsafe trait CeilingLike {}

unsafe impl Ceiling for C1 {
    fn ceiling() -> u8 {
        ((1 << 4) - 1) << 4
    }
}

unsafe impl Ceiling for C2 {
    fn ceiling() -> u8 {
        ((1 << 4) - 2) << 4
    }
}

unsafe impl Ceiling for C3 {
    fn ceiling() -> u8 {
        ((1 << 4) - 3) << 4
    }
}

unsafe impl Ceiling for C4 {
    fn ceiling() -> u8 {
        ((1 << 4) - 4) << 4
    }
}

unsafe impl Ceiling for C5 {
    fn ceiling() -> u8 {
        ((1 << 4) - 5) << 4
    }
}

unsafe impl Ceiling for C6 {
    fn ceiling() -> u8 {
        ((1 << 4) - 6) << 4
    }
}

unsafe impl Ceiling for C7 {
    fn ceiling() -> u8 {
        ((1 << 4) - 7) << 4
    }
}

unsafe impl Ceiling for C8 {
    fn ceiling() -> u8 {
        ((1 << 4) - 8) << 4
    }
}

unsafe impl Ceiling for C9 {
    fn ceiling() -> u8 {
        ((1 << 4) - 9) << 4
    }
}

unsafe impl Ceiling for C10 {
    fn ceiling() -> u8 {
        ((1 << 4) - 10) << 4
    }
}

unsafe impl Ceiling for C11 {
    fn ceiling() -> u8 {
        ((1 << 4) - 11) << 4
    }
}

unsafe impl Ceiling for C12 {
    fn ceiling() -> u8 {
        ((1 << 4) - 12) << 4
    }
}

unsafe impl Ceiling for C13 {
    fn ceiling() -> u8 {
        ((1 << 4) - 13) << 4
    }
}

unsafe impl Ceiling for C14 {
    fn ceiling() -> u8 {
        ((1 << 4) - 14) << 4
    }
}

unsafe impl Ceiling for C15 {
    fn ceiling() -> u8 {
        ((1 << 4) - 15) << 4
    }
}

unsafe impl CeilingLike for C0 {}
unsafe impl CeilingLike for C1 {}
unsafe impl CeilingLike for C2 {}
unsafe impl CeilingLike for C3 {}
unsafe impl CeilingLike for C4 {}
unsafe impl CeilingLike for C5 {}
unsafe impl CeilingLike for C6 {}
unsafe impl CeilingLike for C7 {}
unsafe impl CeilingLike for C8 {}
unsafe impl CeilingLike for C9 {}
unsafe impl CeilingLike for C10 {}
unsafe impl CeilingLike for C11 {}
unsafe impl CeilingLike for C12 {}
unsafe impl CeilingLike for C13 {}
unsafe impl CeilingLike for C14 {}
unsafe impl CeilingLike for C15 {}
