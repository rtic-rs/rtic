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
use cortex_m::interrupt::CriticalSection;
use cortex_m::peripheral::Peripheral;
use cortex_m::register::{basepri, basepri_max};

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr;

// NOTE Only the 4 highest bits of the priority byte (BASEPRI / NVIC.IPR) are
// considered when determining priorities.
const PRIORITY_BITS: u8 = 4;

// XXX Do we need memory / instruction / compiler barriers here?
#[inline(always)]
unsafe fn lock<T, R, C, F>(f: F, res: *const T, ceiling: u8) -> R
where
    C: Ceiling,
    F: FnOnce(&T, C) -> R,
{
    let old_basepri = basepri::read();
    basepri_max::write(ceiling);
    let ret = f(&*res, ptr::read(0 as *const _));
    basepri::write(old_basepri);
    ret
}

// XXX Do we need memory / instruction / compiler barriers here?
#[inline(always)]
unsafe fn lock_mut<T, R, C, F>(f: F, res: *mut T, ceiling: u8) -> R
where
    C: Ceiling,
    F: FnOnce(&mut T, C) -> R,
{
    let old_basepri = basepri::read();
    basepri_max::write(ceiling);
    let ret = f(&mut *res, ptr::read(0 as *const _));
    basepri::write(old_basepri);
    ret
}

/// A peripheral as a resource
pub struct ResourceP<P, Ceiling>
where
    P: 'static,
{
    _marker: PhantomData<Ceiling>,
    peripheral: Peripheral<P>,
}

impl<P, C> ResourceP<P, C>
where
    C: CeilingLike,
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

    /// Borrows the resource for the duration of `interrupt::free`
    pub fn cs_borrow<'cs>(&self, _ctxt: &'cs CriticalSection) -> &'cs P {
        unsafe { &*self.peripheral.get() }
    }

    /// Mutably borrows the resource for the duration of `interrupt::free`
    pub fn cs_borrow_mut<'cs>(
        &self,
        _ctxt: &'cs mut CriticalSection,
    ) -> &'cs mut P {
        unsafe { &mut *self.peripheral.get() }
    }
}

impl<P, C> ResourceP<P, C>
where
    C: Ceiling,
{
    /// Borrows the resource without locking
    ///
    /// NOTE The system ceiling must be higher than this resource ceiling
    pub fn borrow<'l, SC>(&'static self, _system_ceiling: &'l SC) -> &'l P
    where
        SC: HigherThan<C>,
    {
        unsafe { &*self.peripheral.get() }
    }

    /// Mutably borrows the resource without locking
    ///
    /// NOTE The system ceiling must be higher than this resource ceiling
    pub fn borrow_mut<'l, SC>(
        &'static self,
        _system_ceiling: &'l mut SC,
    ) -> &'l mut P
    where
        SC: HigherThan<C>,
    {
        unsafe { &mut *self.peripheral.get() }
    }

    /// Returns a mutable pointer to the wrapped value
    pub fn get(&self) -> *mut P {
        self.peripheral.get()
    }

    /// Locks the resource, preventing tasks with priority lower than `Ceiling`
    /// from preempting the current task
    pub fn lock<R, F, Ctxt>(&'static self, _ctxt: &Ctxt, f: F) -> R
    where
        F: FnOnce(&P, C) -> R,
        Ctxt: Context,
    {
        unsafe { lock(f, self.peripheral.get(), C::ceiling()) }
    }

    /// Mutably locks the resource, preventing tasks with priority lower than
    /// `Ceiling` from preempting the current task
    pub fn lock_mut<R, F, Ctxt>(&'static self, _ctxt: &mut Ctxt, f: F) -> R
    where
        F: FnOnce(&mut P, C) -> R,
        Ctxt: Context,
    {
        unsafe { lock_mut(f, self.peripheral.get(), C::ceiling()) }
    }
}

impl<P> ResourceP<P, C0> {
    /// Borrows the resource without locking
    pub fn borrow<'ctxt, Ctxt>(&'static self, _ctxt: &'ctxt Ctxt) -> &'ctxt P
    where
        Ctxt: Context,
    {
        unsafe { &*self.peripheral.get() }
    }

    /// Mutably borrows the resource without locking
    pub fn borrow_mut<'ctxt, Ctxt>(
        &'static self,
        _ctxt: &'ctxt mut Ctxt,
    ) -> &'ctxt mut P
    where
        Ctxt: Context,
    {
        unsafe { &mut *self.peripheral.get() }
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
    where
        C: CeilingLike,
    {
        Resource {
            _marker: PhantomData,
            data: UnsafeCell::new(data),
        }
    }

    /// Borrows the resource for the duration of `interrupt::free`
    pub fn cs_borrow<'cs>(&self, _ctxt: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.data.get() }
    }

    /// Mutably borrows the resource for the duration of `interrupt::free`
    pub fn cs_borrow_mut<'cs>(
        &self,
        _ctxt: &'cs mut CriticalSection,
    ) -> &'cs mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T, C> Resource<T, C>
where
    C: Ceiling,
{
    /// Borrows the resource without locking
    ///
    /// NOTE The system ceiling must be higher than this resource ceiling
    pub fn borrow<'l, SC>(&'static self, _system_ceiling: &'l SC) -> &'l T
    where
        SC: HigherThan<C>,
    {
        unsafe { &*self.data.get() }
    }

    /// Mutably borrows the resource without locking
    ///
    /// NOTE The system ceiling must be higher than this resource ceiling
    pub fn borrow_mut<'l, SC>(&'static self, _ctxt: &'l mut SC) -> &'l mut T
    where
        SC: HigherThan<C>,
    {
        unsafe { &mut *self.data.get() }
    }

    /// Returns a mutable pointer to the wrapped value
    pub fn get(&self) -> *mut T {
        self.data.get()
    }

    /// Locks the resource, preventing tasks with priority lower than `Ceiling`
    /// from preempting the current task
    pub fn lock<F, R, Ctxt>(&'static self, _ctxt: &Ctxt, f: F) -> R
    where
        F: FnOnce(&T, C) -> R,
        Ctxt: Context,
    {
        unsafe { lock(f, self.data.get(), C::ceiling()) }
    }

    /// Mutably locks the resource, preventing tasks with priority lower than
    /// `Ceiling` from preempting the current task
    pub fn lock_mut<F, R, Ctxt>(&'static self, _ctxt: &mut Ctxt, f: F) -> R
    where
        F: FnOnce(&mut T, C) -> R,
        Ctxt: Context,
    {
        unsafe { lock_mut(f, self.data.get(), C::ceiling()) }
    }
}

impl<T> Resource<T, C0> {
    /// Borrows the resource without locking
    pub fn borrow<'ctxt, Ctxt>(&'static self, _ctxt: &'ctxt Ctxt) -> &'ctxt T
    where
        Ctxt: Context,
    {
        unsafe { &*self.data.get() }
    }

    /// Mutably borrows the resource without locking
    pub fn borrow_mut<'ctxt, Ctxt>(
        &'static self,
        _ctxt: &'ctxt mut Ctxt,
    ) -> &'ctxt mut T
    where
        Ctxt: Context,
    {
        unsafe { &mut *self.data.get() }
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
/// NOTE Input `priority` must be in the range `[1, 16]` (inclusive)
pub fn logical(priority: u8) -> u8 {
    assert!(priority >= 1 && priority <= 16);

    ((1 << PRIORITY_BITS) - priority) << (8 - PRIORITY_BITS)
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
// XXX this should be a "closed" trait
pub unsafe trait CeilingLike {}

/// This ceiling is lower than `C`
// XXX this should be a "closed" trait
pub unsafe trait HigherThan<C> {}

unsafe impl HigherThan<C1> for C2 {}
unsafe impl HigherThan<C1> for C3 {}
unsafe impl HigherThan<C1> for C4 {}
unsafe impl HigherThan<C1> for C5 {}
unsafe impl HigherThan<C1> for C6 {}
unsafe impl HigherThan<C1> for C7 {}
unsafe impl HigherThan<C1> for C8 {}
unsafe impl HigherThan<C1> for C9 {}
unsafe impl HigherThan<C1> for C10 {}
unsafe impl HigherThan<C1> for C11 {}
unsafe impl HigherThan<C1> for C12 {}
unsafe impl HigherThan<C1> for C13 {}
unsafe impl HigherThan<C1> for C14 {}
unsafe impl HigherThan<C1> for C15 {}

unsafe impl HigherThan<C2> for C3 {}
unsafe impl HigherThan<C2> for C4 {}
unsafe impl HigherThan<C2> for C5 {}
unsafe impl HigherThan<C2> for C6 {}
unsafe impl HigherThan<C2> for C7 {}
unsafe impl HigherThan<C2> for C8 {}
unsafe impl HigherThan<C2> for C9 {}
unsafe impl HigherThan<C2> for C10 {}
unsafe impl HigherThan<C2> for C11 {}
unsafe impl HigherThan<C2> for C12 {}
unsafe impl HigherThan<C2> for C13 {}
unsafe impl HigherThan<C2> for C14 {}
unsafe impl HigherThan<C2> for C15 {}

unsafe impl HigherThan<C3> for C4 {}
unsafe impl HigherThan<C3> for C5 {}
unsafe impl HigherThan<C3> for C6 {}
unsafe impl HigherThan<C3> for C7 {}
unsafe impl HigherThan<C3> for C8 {}
unsafe impl HigherThan<C3> for C9 {}
unsafe impl HigherThan<C3> for C10 {}
unsafe impl HigherThan<C3> for C11 {}
unsafe impl HigherThan<C3> for C12 {}
unsafe impl HigherThan<C3> for C13 {}
unsafe impl HigherThan<C3> for C14 {}
unsafe impl HigherThan<C3> for C15 {}

unsafe impl HigherThan<C4> for C5 {}
unsafe impl HigherThan<C4> for C6 {}
unsafe impl HigherThan<C4> for C7 {}
unsafe impl HigherThan<C4> for C8 {}
unsafe impl HigherThan<C4> for C9 {}
unsafe impl HigherThan<C4> for C10 {}
unsafe impl HigherThan<C4> for C11 {}
unsafe impl HigherThan<C4> for C12 {}
unsafe impl HigherThan<C4> for C13 {}
unsafe impl HigherThan<C4> for C14 {}
unsafe impl HigherThan<C4> for C15 {}

unsafe impl HigherThan<C5> for C6 {}
unsafe impl HigherThan<C5> for C7 {}
unsafe impl HigherThan<C5> for C8 {}
unsafe impl HigherThan<C5> for C9 {}
unsafe impl HigherThan<C5> for C10 {}
unsafe impl HigherThan<C5> for C11 {}
unsafe impl HigherThan<C5> for C12 {}
unsafe impl HigherThan<C5> for C13 {}
unsafe impl HigherThan<C5> for C14 {}
unsafe impl HigherThan<C5> for C15 {}

unsafe impl HigherThan<C6> for C7 {}
unsafe impl HigherThan<C6> for C8 {}
unsafe impl HigherThan<C6> for C9 {}
unsafe impl HigherThan<C6> for C10 {}
unsafe impl HigherThan<C6> for C11 {}
unsafe impl HigherThan<C6> for C12 {}
unsafe impl HigherThan<C6> for C13 {}
unsafe impl HigherThan<C6> for C14 {}
unsafe impl HigherThan<C6> for C15 {}

unsafe impl HigherThan<C7> for C8 {}
unsafe impl HigherThan<C7> for C9 {}
unsafe impl HigherThan<C7> for C10 {}
unsafe impl HigherThan<C7> for C11 {}
unsafe impl HigherThan<C7> for C12 {}
unsafe impl HigherThan<C7> for C13 {}
unsafe impl HigherThan<C7> for C14 {}
unsafe impl HigherThan<C7> for C15 {}

unsafe impl HigherThan<C8> for C9 {}
unsafe impl HigherThan<C8> for C10 {}
unsafe impl HigherThan<C8> for C11 {}
unsafe impl HigherThan<C8> for C12 {}
unsafe impl HigherThan<C8> for C13 {}
unsafe impl HigherThan<C8> for C14 {}
unsafe impl HigherThan<C8> for C15 {}

unsafe impl HigherThan<C9> for C10 {}
unsafe impl HigherThan<C9> for C11 {}
unsafe impl HigherThan<C9> for C12 {}
unsafe impl HigherThan<C9> for C13 {}
unsafe impl HigherThan<C9> for C14 {}
unsafe impl HigherThan<C9> for C15 {}

unsafe impl HigherThan<C10> for C11 {}
unsafe impl HigherThan<C10> for C12 {}
unsafe impl HigherThan<C10> for C13 {}
unsafe impl HigherThan<C10> for C14 {}
unsafe impl HigherThan<C10> for C15 {}

unsafe impl HigherThan<C11> for C12 {}
unsafe impl HigherThan<C11> for C13 {}
unsafe impl HigherThan<C11> for C14 {}
unsafe impl HigherThan<C11> for C15 {}

unsafe impl HigherThan<C12> for C13 {}
unsafe impl HigherThan<C12> for C14 {}
unsafe impl HigherThan<C12> for C15 {}

unsafe impl HigherThan<C13> for C14 {}
unsafe impl HigherThan<C13> for C15 {}

unsafe impl HigherThan<C14> for C15 {}

unsafe impl Ceiling for C1 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 1) << 4
    }
}

unsafe impl Ceiling for C2 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 2) << 4
    }
}

unsafe impl Ceiling for C3 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 3) << 4
    }
}

unsafe impl Ceiling for C4 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 4) << 4
    }
}

unsafe impl Ceiling for C5 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 5) << 4
    }
}

unsafe impl Ceiling for C6 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 6) << 4
    }
}

unsafe impl Ceiling for C7 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 7) << 4
    }
}

unsafe impl Ceiling for C8 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 8) << 4
    }
}

unsafe impl Ceiling for C9 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 9) << 4
    }
}

unsafe impl Ceiling for C10 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 10) << 4
    }
}

unsafe impl Ceiling for C11 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 11) << 4
    }
}

unsafe impl Ceiling for C12 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 12) << 4
    }
}

unsafe impl Ceiling for C13 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 13) << 4
    }
}

unsafe impl Ceiling for C14 {
    #[inline(always)]
    fn ceiling() -> u8 {
        ((1 << 4) - 14) << 4
    }
}

unsafe impl Ceiling for C15 {
    #[inline(always)]
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
