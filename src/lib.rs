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
use cortex_m::interrupt::{CriticalSection, Nr};
use cortex_m::peripheral::{Peripheral, NVIC, SCB};
use cortex_m::register::{basepri, basepri_max};

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr;

// NOTE Only the 4 highest bits of the priority byte (BASEPRI / NVIC.IPR) are
// considered when determining priorities.
const PRIORITY_BITS: u8 = 4;

/// Logical task priority
unsafe fn task_priority() -> u8 {
    // NOTE(safe) atomic read
    let nr = match (*SCB.get()).icsr.read() as u8 {
        n if n >= 16 => n - 16,
        _ => panic!("not in a task"),
    };
    // NOTE(safe) atomic read
    hardware((*NVIC.get()).ipr[nr as usize].read())
}

#[cfg(debug_assertions)]
unsafe fn lock_check(ceiling: u8) {
    let ceiling = hardware(ceiling);
    let task_priority = task_priority();

    if task_priority > ceiling {
        panic!(
            "bad ceiling value. task_priority = {}, resource_ceiling = {}",
            task_priority,
            ceiling,
        );
    }
}

#[cfg(not(debug_assertions))]
unsafe fn lock_check(_ceiling: u8) {}

// XXX Do we need memory / instruction / compiler barriers here?
#[inline(always)]
unsafe fn lock<T, R, C, F>(f: F, res: *const T, ceiling: u8) -> R
where
    C: Ceiling,
    F: FnOnce(&T, C) -> R,
{
    lock_check(ceiling);
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
    lock_check(ceiling);
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
        unsafe { lock(f, self.peripheral.get(), C::hw_ceiling()) }
    }

    /// Mutably locks the resource, preventing tasks with priority lower than
    /// `Ceiling` from preempting the current task
    pub fn lock_mut<R, F, Ctxt>(&'static self, _ctxt: &mut Ctxt, f: F) -> R
    where
        F: FnOnce(&mut P, C) -> R,
        Ctxt: Context,
    {
        unsafe { lock_mut(f, self.peripheral.get(), C::hw_ceiling()) }
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

    /// Returns a mutable reference to the wrapped value
    pub unsafe fn get(&self) -> &'static mut T {
        match () {
            #[cfg(debug_assertions)]
            () => {
                let task_priority = task_priority();
                let system_ceiling =
                    hardware(cortex_m::register::basepri::read());
                let resource_ceiling = C::ceiling();

                if resource_ceiling < task_priority {
                    panic!("bad ceiling value. task priority = {}, \
                            resource ceiling = {}",
                           task_priority,
                           resource_ceiling);
                } else if resource_ceiling == task_priority {
                    // OK: safe to access the resource without locking in the
                    // task with highest priority
                } else if resource_ceiling <= system_ceiling {
                    // OK: use within another resource critical section, where
                    // the locked resource has higher or equal ceiling
                } else {
                    panic!("racy access to resource. \
                            task priority = {}, \
                            resource ceiling = {}, \
                            system ceiling = {}",
                           task_priority,
                           resource_ceiling,
                           system_ceiling);
                }

            }
            #[cfg(not(debug_assertions))]
            () => {}
        }

        &mut *self.data.get()
    }

    /// Locks the resource, preventing tasks with priority lower than `Ceiling`
    /// from preempting the current task
    pub fn lock<F, R, Ctxt>(&'static self, _ctxt: &Ctxt, f: F) -> R
    where
        F: FnOnce(&T, C) -> R,
        Ctxt: Context,
    {
        unsafe { lock(f, self.data.get(), C::hw_ceiling()) }
    }

    /// Mutably locks the resource, preventing tasks with priority lower than
    /// `Ceiling` from preempting the current task
    pub fn lock_mut<F, R, Ctxt>(&'static self, _ctxt: &mut Ctxt, f: F) -> R
    where
        F: FnOnce(&mut T, C) -> R,
        Ctxt: Context,
    {
        unsafe { lock_mut(f, self.data.get(), C::hw_ceiling()) }
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

/// Maps a hardware priority to a logical priority
fn hardware(priority: u8) -> u8 {
    16 - (priority >> (8 - PRIORITY_BITS))
}

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

/// Puts `interrupt` in the "to execute" queue
///
/// This function has no effect if the interrupt was already queued
pub fn queue<I>(interrupt: I) where I: Nr {
    unsafe {
        // NOTE(safe) atomic write
        (*NVIC.get()).set_pending(interrupt)
    }
}

/// Fake ceiling, indicates that the resource is shared by cooperative tasks
pub struct C0 {
    _0: (),
}

/// A real ceiling
// XXX this should be a "closed" trait
pub unsafe trait Ceiling {
    /// Returns the logical ceiling as a number
    fn ceiling() -> u8;

    /// Returns the HW ceiling as a number
    fn hw_ceiling() -> u8;
}

/// Usable as a ceiling
// XXX this should be a "closed" trait
pub unsafe trait CeilingLike {}

/// This ceiling is lower than `C`
// XXX this should be a "closed" trait
pub unsafe trait HigherThan<C> {}

macro_rules! ceiling {
    ($ceiling:ident, $logical:expr) => {
        /// Ceiling
        pub struct $ceiling {
            _0: ()
        }

        unsafe impl CeilingLike for $ceiling {}

        unsafe impl Ceiling for $ceiling {
            #[inline(always)]
            fn ceiling() -> u8 {
                $logical
            }

            #[inline(always)]
            fn hw_ceiling() -> u8 {
                ((1 << PRIORITY_BITS) - $logical) << (8 - PRIORITY_BITS)
            }
        }
    }
}

ceiling!(C1, 1);
ceiling!(C2, 2);
ceiling!(C3, 3);
ceiling!(C4, 4);
ceiling!(C5, 5);
ceiling!(C6, 6);
ceiling!(C7, 7);
ceiling!(C8, 8);
ceiling!(C9, 9);
ceiling!(C10, 10);
ceiling!(C11, 11);
ceiling!(C12, 12);
ceiling!(C13, 13);
ceiling!(C14, 14);
ceiling!(C15, 15);

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

unsafe impl CeilingLike for C0 {}
