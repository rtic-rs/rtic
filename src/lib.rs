//! Stack Resource Policy

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
extern crate static_ref;
extern crate typenum;

use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;

use cortex_m::ctxt::Context;
use cortex_m::interrupt::Nr;
#[cfg(not(thumbv6m))]
use cortex_m::register::{basepri, basepri_max};
use static_ref::Ref;
use typenum::{Cmp, Unsigned};
#[cfg(not(thumbv6m))]
use typenum::{Greater, Less};

pub use cortex_m::ctxt::Local;
pub use cortex_m::asm::{bkpt, wfi};

#[doc(hidden)]
pub use cortex_m::peripheral::NVIC;

use typenum::type_operators::*;

macro_rules! barrier {
    () => {
        asm!(""
             :
             :
             : "memory"
             : "volatile");
    }
}

/// A resource
pub struct Resource<T, CEILING> {
    _ceiling: PhantomData<CEILING>,
    data: UnsafeCell<T>,
}

impl<T, C> Resource<T, C> {
    /// Creates a new resource with ceiling `C`
    pub const fn new(data: T) -> Self
    where
        C: Ceiling,
    {
        Resource {
            _ceiling: PhantomData,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T, CEILING> Resource<T, C<CEILING>> {
    /// Borrows the resource for the duration of another resource's critical
    /// section
    ///
    /// This operation is zero cost and doesn't impose any additional blocking
    pub fn borrow<'cs, PRIORITY, SCEILING>(
        &'static self,
        _priority: &P<PRIORITY>,
        _system_ceiling: &'cs C<SCEILING>,
    ) -> Ref<'cs, T>
    where
        SCEILING: GreaterThanOrEqual<CEILING>,
        CEILING: GreaterThanOrEqual<PRIORITY>,
    {
        unsafe { Ref::new(&*self.data.get()) }
    }

    /// Locks the resource for the duration of the critical section `f`
    ///
    /// For the duration of the critical section, tasks whose priority level is
    /// smaller than or equal to the resource `CEILING` will be prevented from
    /// preempting the current task.
    ///
    /// Within this critical section, resources with ceiling equal to or smaller
    /// than `CEILING` can be borrowed at zero cost. See
    /// [Resource.borrow](struct.Resource.html#method.borrow).
    #[cfg(not(thumbv6m))]
    pub fn lock<R, PRIORITY, F>(&'static self, _priority: &P<PRIORITY>, f: F) -> R
        where F: FnOnce(Ref<T>, &C<CEILING>) -> R,
              CEILING: Cmp<PRIORITY, Output = Greater> + Cmp<UMAX, Output = Less> + Level
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(<CEILING>::hw());
            barrier!();
            let ret =
                f(Ref::new(&*self.data.get()), &C { _marker: PhantomData });
            barrier!();
            basepri::write(old_basepri);
            ret
        }
    }

    /// Locks the resource for the duration of the critical section `f`
    ///
    /// For the duration of the critical section, tasks whose priority level is
    /// smaller than or equal to the resource `CEILING` will be prevented from
    /// preempting the current task.
    ///
    /// Within this critical section, resources with ceiling equal to or smaller
    /// than `CEILING` can be borrowed at zero cost. See
    /// [Resource.borrow](struct.Resource.html#method.borrow).
    #[cfg(not(thumbv6m))]
    pub fn mock<R, PRIOTASK, CURRCEIL, F>(
        &'static self,
        _prio: &P<PRIOTASK>,
        _curr_ceil: &C<CURRCEIL>,
        f: F,
    ) -> R
    where
        F: FnOnce(Ref<T>, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
        PRIOTASK: Unsigned,
        CURRCEIL: Unsigned,
        CEILING: GreaterThanOrEqual<PRIOTASK> + Max<CURRCEIL> + Level + Unsigned,
    {
        unsafe {
            let c1 = <CURRCEIL>::to_u8();
            let c2 = <CEILING>::to_u8();
            if c2 > c1 {
                let old_basepri = basepri::read();
                basepri_max::write(<CEILING>::hw());
                barrier!();
                let ret =
                    f(Ref::new(&*self.data.get()), &C { _marker: PhantomData });
                barrier!();
                basepri::write(old_basepri);
                ret
            } else {
                f(Ref::new(&*self.data.get()), &C { _marker: PhantomData })

            }
        }
    }

    /// Locks the resource for the duration of the critical section `f`
    ///
    /// For the duration of the critical section, tasks whose priority level is
    /// smaller than or equal to the resource `CEILING` will be prevented from
    /// preempting the current task.
    ///
    /// Within this critical section, resources with ceiling equal to or smaller
    /// than `CEILING` can be borrowed at zero cost. See
    /// [Resource.borrow](struct.Resource.html#method.borrow).
    #[cfg(not(thumbv6m))]
    pub fn claim<R, PRIOTASK, CURRCEIL, F>(
        &'static self,
        _prio: &P<PRIOTASK>,
        _curr_ceil: &C<CURRCEIL>,
        f: F,
    ) -> R
    where
        F: FnOnce(Ref<T>, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
        PRIOTASK: Unsigned,
        CURRCEIL: Unsigned,
        CEILING: GreaterThanOrEqual<PRIOTASK> + Max<CURRCEIL> + Level + Unsigned,
    {
        unsafe {
            let c1 = <CURRCEIL>::to_u8();
            let c2 = <CEILING>::to_u8();
            if c2 > c1 {
                let old_basepri = basepri::read();
                basepri_max::write(<CEILING>::hw());
                barrier!();
                let ret =
                    f(Ref::new(&*self.data.get()), &C { _marker: PhantomData });
                barrier!();
                basepri::write(old_basepri);
                ret
            } else {
                f(Ref::new(&*self.data.get()), &C { _marker: PhantomData })

            }
        }
    }
}

unsafe impl<T, C> Sync for Resource<T, C>
where
    C: Ceiling,
{
}

// re-implementation of the original claim API
/// A resource
pub struct Res<T, CEILING> {
    _ceiling: PhantomData<CEILING>,
    _state: Cell<bool>,
    data: UnsafeCell<T>,
}

impl<T, C> Res<T, C> {
    /// Creates a new resource with ceiling `C`
    pub const fn new(data: T) -> Self
    where
        C: Ceiling,
    {
        Res {
            _ceiling: PhantomData,
            _state: Cell::new(true),
            //    _state: State::Free,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T, CEILING> Res<T, C<CEILING>> {
    /// Locks the resource for the duration of the critical section `f`
    ///
    /// For the duration of the critical section, tasks whose priority level is
    /// smaller than or equal to the resource `CEILING` will be prevented from
    /// preempting the current task.
    ///
    /// claim takes three args of type R, PRIOTASK, CURRCEIL, F
    /// R 				is the Resource to lock (self)
    /// PRIOTASK 		is the priority of the task (context) calling claim
    /// CURRCEIL		is the current system ceiling
    ///
    /// F is the type of the closure, hande &T and &C
    /// &T 				is a read only reference to the data
    /// &C 				is the new system ceiling
    ///
    /// Usage example: a task at prio P1, claiming R1 and R2.
    /// fn j(_task: Exti0, p: P1) {
    ///     R1.claim_mut(
    ///         &p, &p.as_ceiling(), |a, c| {
    ///             R2.claim_mut(
    ///                 &p, &c, |b, _| {
    ///                     b.y = a[0]; // b is mutable
    ///                     a[1] = b.y; // a is mutable
    ///                 }
    ///             );
    ///             a[2] = 0; // a is mutable
    ///         }
    ///     );
    /// }
    /// The implementation satisfies the following
    /// 1. Race free access to the resources under the Stack Resource Policy (SRP)
    /// System ceiling SC, is implemented by the NVIC as MAX(TASKPRI, BASEPRPI)
    /// Hence in case TASKPRI = SC, the BASEPRI register does not need to be updated
    /// as checked by c2 > c1 (this an optimization)
    ///
    /// 2. Static (compile time verification) that SC >= TASKPRI
    /// This is achieved by the type bound CEILING: GreaterThanOrEqual<TASKPRI>
    /// This gives ensurence that no task access a resoure with lower ceiling value than the task
    /// and hence satisfies the soundness rule of SRP
    ///
    /// 3. The system ceileng for the closure CS = MAX(CURRCEIL, R)
    /// This is achieved by &C<<CEILING as Max<CURRCEIL>>::Output>
    /// where Max operates on R and CEILING
    ///
    /// 4. Rust aliasing rules are ensured as run-time check raises a panic if resourse state is locked
    /// This resembles the RefCell implementation, but the implementation is more strict as multiple
    /// readers are disallowed. Essentially this forbids re-locking
    ///
    /// Usage example failing: a task at prio P1, claiming R1 and R1.
    /// fn j(_task: Exti0, p: P1) {
    ///     R1.claim_mut(
    ///         &p, &p.as_ceiling(), |a1, c| {
    ///             R1.claim_mut(  <-- at this point a panic will occur
    ///                 &p, &c, |a2, _| {
    ///                 }
    ///             );
    ///             a1[2] = 0; // a is mutable
    ///         }
    ///     );
    /// }
    ///
    /// 5. The ceiling of the closure cannot be leaked
    ///
    /// fn jres_opt_leak(_task: Exti0, p: P1) {
    ///     R1.claim_mut(
    ///         &p, &p.as_ceiling(), |a, c| {
    ///             let leak_c = R2.claim_mut(&p, &c, |b, c| c); <-- trying to leak c as a return value
    ///
    ///             R5.claim_mut(&p, leak_c, |b, c| {}); <-- trying to use a leaked system ceilng
    ///
    ///             a[2] = 0;
    ///         }
    ///     );
    /// }
    ///
    /// The compiler will reject leakage due to the lifetime, (c in a closure is a &C, so it cannot be returned)
    /// A leakage would indeed be fatal as claim would hand out an unprotected R
    #[cfg(not(thumbv6m))]
    pub fn claim_mut<R, TASKPRI, CURRCEIL, F>(
        &'static self,
        _prio: &P<TASKPRI>,
        _curr_ceil: &C<CURRCEIL>,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut T, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
        TASKPRI: Unsigned,
        CURRCEIL: Unsigned,
        CEILING: GreaterThanOrEqual<TASKPRI> + Max<CURRCEIL> + Level + Unsigned,
    {
        if self._state.get() {
            unsafe {
                let curr_ceil = <CURRCEIL>::to_u8();
                let new_ceil = <CEILING>::to_u8();
                if new_ceil > curr_ceil {
                    let old_basepri = basepri::read();
                    basepri_max::write(<CEILING>::hw());
                    barrier!();
                    self._state.set(false);
                    barrier!();

                    let ret =
                        f(&mut *self.data.get(), &C { _marker: PhantomData });

                    barrier!();
                    self._state.set(true);
                    barrier!();
                    basepri::write(old_basepri);
                    ret
                } else {
                    self._state.set(false);
                    barrier!();

                    let ret =
                        f(&mut *self.data.get(), &C { _marker: PhantomData });

                    barrier!();
                    self._state.set(true);
                    ret
                }
            }
        } else {
            panic!("Resource already locked)")
        }
    }

    /// Read only claim, see claim_mut above
    #[cfg(not(thumbv6m))]
    pub fn claim<R, TASKPRI, CURRCEIL, F>(
        &'static self,
        _prio: &P<TASKPRI>,
        _curr_ceil: &C<CURRCEIL>,
        f: F,
    ) -> R
    where
        F: FnOnce(&T, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
        TASKPRI: Unsigned,
        CURRCEIL: Unsigned,
        CEILING: GreaterThanOrEqual<TASKPRI> + Max<CURRCEIL> + Level + Unsigned,
    {
        if self._state.get() {
            unsafe {
                let curr_ceil = <CURRCEIL>::to_u8();
                let new_ceil = <CEILING>::to_u8();
                if new_ceil > curr_ceil {
                    let old_basepri = basepri::read();
                    basepri_max::write(<CEILING>::hw());
                    barrier!();
                    self._state.set(false);
                    barrier!();

                    let ret = f(&*self.data.get(), &C { _marker: PhantomData });

                    barrier!();
                    self._state.set(true);
                    barrier!();
                    basepri::write(old_basepri);
                    ret
                } else {
                    self._state.set(false);
                    barrier!();

                    let ret = f(&*self.data.get(), &C { _marker: PhantomData });

                    barrier!();
                    self._state.set(true);
                    ret
                }
            }
        } else {
            panic!("Resource already locked)")
        }
    }

    /// Unsafe version of claim_mut
    #[cfg(not(thumbv6m))]
    pub unsafe fn claim_mut_unsafe<R, TASKPRI, CURRCEIL, F>(
        &'static self,
        _prio: &P<TASKPRI>,
        _curr_ceil: &C<CURRCEIL>,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut T, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
        TASKPRI: Unsigned,
        CURRCEIL: Unsigned,
        CEILING: GreaterThanOrEqual<TASKPRI> + Max<CURRCEIL> + Level + Unsigned,
    {

        let curr_ceil = <CURRCEIL>::to_u8();
        let new_ceil = <CEILING>::to_u8();
        if new_ceil > curr_ceil {
            let old_basepri = basepri::read();
            basepri_max::write(<CEILING>::hw());
            barrier!();

            let ret = f(&mut *self.data.get(), &C { _marker: PhantomData });

            barrier!();
            basepri::write(old_basepri);
            ret
        } else {

            let ret = f(&mut *self.data.get(), &C { _marker: PhantomData });

            ret
        }
    }
}

unsafe impl<T, C> Sync for Res<T, C>
where
    C: Ceiling,
{
}


/// Nem attempt
use core::cell::RefCell;
use core::cell::RefMut;

//use core::borrow::BorrowMut;
/// A resource
pub struct ResRef<T, CEILING> {
    _ceiling: PhantomData<CEILING>,
    data: UnsafeCell<RefCell<T>>,
}

impl<T, C> ResRef<T, C> {
    /// Creates a new resource with ceiling `C`
    pub const fn new(data: T) -> Self
    where
        C: Ceiling,
    {
        ResRef {
            _ceiling: PhantomData,
            data: UnsafeCell::new(RefCell::new(data)),
        }
    }
}

impl<T, CEILING> ResRef<T, C<CEILING>> {
    /// Locks the resource for the duration of the critical section `f`
    ///
    /// For the duration of the critical section, tasks whose priority level is
    /// smaller than or equal to the resource `CEILING` will be prevented from
    /// preempting the current task.
    ///
    /// claim takes three args of type R, PRIOTASK, CURRCEIL, F
    /// R 				is the Resource to lock (self)
    /// PRIOTASK 		is the priority of the task (context) calling claim
    /// CURRCEIL		is the current system ceiling
    ///
    /// F is the type of the closure, hande &T and &C
    /// &T 				is a read only reference to the data
    /// &C 				is the new system ceiling
    ///
    /// Usage example: a task at prio P1, claiming R1 and R2.
    /// fn j(_task: Exti0, p: P1) {
    ///     R1.claim_mut(
    ///         &p, &p.as_ceiling(), |a, c| {
    ///             R2.claim_mut(
    ///                 &p, &c, |b, _| {
    ///                     b.y = a[0]; // b is mutable
    ///                     a[1] = b.y; // a is mutable
    ///                 }
    ///             );
    ///             a[2] = 0; // a is mutable
    ///         }
    ///     );
    /// }
    /// The implementation satisfies the following
    /// 1. Race free access to the resources under the Stack Resource Policy (SRP)
    /// System ceiling SC, is implemented by the NVIC as MAX(TASKPRI, BASEPRPI)
    /// Hence in case TASKPRI = SC, the BASEPRI register does not need to be updated
    /// as checked by c2 > c1 (this an optimization)
    ///
    /// 2. Static (compile time verification) that SC >= TASKPRI
    /// This is achieved by the type bound CEILING: GreaterThanOrEqual<TASKPRI>
    /// This gives ensurence that no task access a resoure with lower ceiling value than the task
    /// and hence satisfies the soundness rule of SRP
    ///
    /// 3. The system ceileng for the closure CS = MAX(CURRCEIL, R)
    /// This is achieved by &C<<CEILING as Max<CURRCEIL>>::Output>
    /// where Max operates on R and CEILING
    ///
    /// 4. Rust aliasing rules are ensured as run-time check raises a panic if resourse state is locked
    /// This resembles the RefCell implementation, but the implementation is more strict as multiple
    /// readers are disallowed. Essentially this forbids re-locking
    ///
    /// Usage example failing: a task at prio P1, claiming R1 and R1.
    /// fn j(_task: Exti0, p: P1) {
    ///     R1.claim_mut(
    ///         &p, &p.as_ceiling(), |a1, c| {
    ///             R1.claim_mut(  <-- at this point a panic will occur
    ///                 &p, &c, |a2, _| {
    ///                 }
    ///             );
    ///             a1[2] = 0; // a is mutable
    ///         }
    ///     );
    /// }
    ///
    /// 5. The ceiling of the closure cannot be leaked
    ///
    /// fn jres_opt_leak(_task: Exti0, p: P1) {
    ///     R1.claim_mut(
    ///         &p, &p.as_ceiling(), |a, c| {
    ///             let leak_c = R2.claim_mut(&p, &c, |b, c| c); <-- trying to leak c as a return value
    ///
    ///             R5.claim_mut(&p, leak_c, |b, c| {}); <-- trying to use a leaked system ceilng
    ///
    ///             a[2] = 0;
    ///         }
    ///     );
    /// }
    ///
    /// The compiler will reject leakage due to the lifetime, (c in a closure is a &C, so it cannot be returned)
    /// A leakage would indeed be fatal as claim would hand out an unprotected R
    #[cfg(not(thumbv6m))]
    pub fn claim_mut<R, TASKPRI, CURRCEIL, F>(
        &'static self,
        _prio: &P<TASKPRI>,
        _curr_ceil: &C<CURRCEIL>,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut T, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
        TASKPRI: Unsigned,
        CURRCEIL: Unsigned,
        CEILING: GreaterThanOrEqual<TASKPRI> + Max<CURRCEIL> + Level + Unsigned,
    {

        unsafe {
            let curr_ceil = <CURRCEIL>::to_u8();
            let new_ceil = <CEILING>::to_u8();
            if new_ceil > curr_ceil {
                let old_basepri = basepri::read();
                basepri_max::write(<CEILING>::hw());
                barrier!();

                let r: &RefCell<T> = &*self.data.get();
                let rr: RefCell<T> = *r;
                let mut rm: RefMut<T> = rr.borrow_mut();
                let mut t: T = *rm;
                let ret = f(&mut t, &C { _marker: PhantomData });

                barrier!();
                basepri::write(old_basepri);
                ret
            } else {
                panic!("");
            }
        }
    }
}

//
//    /// Read only claim, see claim_mut above
//    #[cfg(not(thumbv6m))]
//    pub fn claim<R, TASKPRI, CURRCEIL, F>(
//        &'static self,
//        _prio: &P<TASKPRI>,
//        _curr_ceil: &C<CURRCEIL>,
//        f: F,
//    ) -> R
//    where
//        F: FnOnce(&T, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
//        TASKPRI: Unsigned,
//        CURRCEIL: Unsigned,
//        CEILING: GreaterThanOrEqual<TASKPRI> + Max<CURRCEIL> + Level + Unsigned,
//    {
//        if self._state.get() {
//            unsafe {
//                let curr_ceil = <CURRCEIL>::to_u8();
//                let new_ceil = <CEILING>::to_u8();
//                if new_ceil > curr_ceil {
//                    let old_basepri = basepri::read();
//                    basepri_max::write(<CEILING>::hw());
//                    barrier!();
//                    self._state.set(false);
//                    barrier!();
//
//                    let ret = f(&*self.data.get(), &C { _marker: PhantomData });
//
//                    barrier!();
//                    self._state.set(true);
//                    barrier!();
//                    basepri::write(old_basepri);
//                    ret
//                } else {
//                    self._state.set(false);
//                    barrier!();
//
//                    let ret = f(&*self.data.get(), &C { _marker: PhantomData });
//
//                    barrier!();
//                    self._state.set(true);
//                    ret
//                }
//            }
//        } else {
//            panic!("Resource already locked)")
//        }
//    }
//
//    /// Unsafe version of claim_mut
//    #[cfg(not(thumbv6m))]
//    pub unsafe fn claim_mut_unsafe<R, TASKPRI, CURRCEIL, F>(
//        &'static self,
//        _prio: &P<TASKPRI>,
//        _curr_ceil: &C<CURRCEIL>,
//        f: F,
//    ) -> R
//    where
//        F: FnOnce(&mut T, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
//        TASKPRI: Unsigned,
//        CURRCEIL: Unsigned,
//        CEILING: GreaterThanOrEqual<TASKPRI> + Max<CURRCEIL> + Level + Unsigned,
//    {
//
//        let curr_ceil = <CURRCEIL>::to_u8();
//        let new_ceil = <CEILING>::to_u8();
//        if new_ceil > curr_ceil {
//            let old_basepri = basepri::read();
//            basepri_max::write(<CEILING>::hw());
//            barrier!();
//
//            let ret = f(&mut *self.data.get(), &C { _marker: PhantomData });
//
//            barrier!();
//            basepri::write(old_basepri);
//            ret
//        } else {
//
//            let ret = f(&mut *self.data.get(), &C { _marker: PhantomData });
//
//            ret
//        }
//    }
//
//
//    //    / Locks the resource for the duration of the critical section `f`
//    //    /
//    //    / For the duration of the critical section, tasks whose priority level is
//    //    / smaller than or equal to the resource `CEILING` will be prevented from
//    //    / preempting the current task.
//    //    /
//    //    / Within this critical section, resources with ceiling equal to or smaller
//    //    / than `CEILING` can be borrowed at zero cost. See
//    //    / [Resource.borrow](struct.Resource.html#method.borrow).
//    //    #[cfg(not(thumbv6m))]
//    //    pub fn claim_mut<R, PRIOTASK, CURRCEIL, F>(
//    //        &'static self,
//    //        _prio: &P<PRIOTASK>,
//    //        _curr_ceil: &C<CURRCEIL>,
//    //        f: F,
//    //    ) -> R
//    //    where
//    //        F: FnOnce(&mut T, &C<<CEILING as Max<CURRCEIL>>::Output>) -> R,
//    //        PRIOTASK: Unsigned,
//    //        CURRCEIL: Unsigned,
//    //        CEILING: GreaterThanOrEqual<PRIOTASK> + Max<CURRCEIL> + Level + Unsigned,
//    //    {
//    //        unsafe {
//    //            match self._state.get() {
//    //                State::Free => {
//    //                    let c1 = <CURRCEIL>::to_u8();
//    //                    let c2 = <CEILING>::to_u8();
//    //                    if c2 > c1 {
//    //                        let old_basepri = basepri::read();
//    //                        basepri_max::write(<CEILING>::hw());
//    //                        barrier!();
//    //                        self._state.set(State::LockedMut);
//    //                        barrier!();
//    //
//    //                        let ret = f(
//    //                            &mut *self.data.get(),
//    //                            &C { _marker: PhantomData },
//    //                        );
//    //
//    //                        barrier!();
//    //                        self._state.set(State::Free);
//    //                        barrier!();
//    //                        basepri::write(old_basepri);
//    //                        ret
//    //                    } else {
//    //                        self._state.set(State::LockedMut);
//    //                        barrier!();
//    //
//    //                        let ret = f(
//    //                            &mut *self.data.get(),
//    //                            &C { _marker: PhantomData },
//    //                        );
//    //
//    //                        barrier!();
//    //                        self._state.set(State::Free);
//    //                        ret
//    //
//    //                    }
//    //                }
//    //                _ => panic!("Resource already locked)"),
//    //            }
//    //        }
//    //    }
//}
//
unsafe impl<T, C> Sync for ResRef<T, C>
where
    C: Ceiling,
{
}


/// A hardware peripheral as a resource
pub struct Peripheral<P, CEILING>
where
    P: 'static,
{
    peripheral: cortex_m::peripheral::Peripheral<P>,
    _ceiling: PhantomData<CEILING>,
}

impl<P, C> Peripheral<P, C>
where
    C: Ceiling,
{
    /// Assigns a ceiling `C` to the `peripheral`
    ///
    /// # Safety
    ///
    /// You MUST not create two resources that point to the same peripheral
    pub const unsafe fn new(peripheral: cortex_m::peripheral::Peripheral<P>,)
        -> Self {
        Peripheral {
            _ceiling: PhantomData,
            peripheral: peripheral,
        }
    }
}

impl<Periph, CEILING> Peripheral<Periph, C<CEILING>> {
    /// See [Resource.borrow](./struct.Resource.html#method.borrow)
    pub fn borrow<'cs, PRIORITY, SCEILING>(
        &'static self,
        _priority: &P<PRIORITY>,
        _system_ceiling: &'cs C<SCEILING>,
    ) -> Ref<'cs, Periph>
    where
        SCEILING: GreaterThanOrEqual<CEILING>,
        CEILING: GreaterThanOrEqual<PRIORITY>,
    {
        unsafe { Ref::new(&*self.peripheral.get()) }
    }

    /// See [Resource.lock](./struct.Resource.html#method.lock)
    #[cfg(not(thumbv6m))]
    pub fn lock<R, PRIORITY, F>(&'static self, _priority: &P<PRIORITY>, f: F) -> R
        where F: FnOnce(Ref<Periph>, &C<CEILING>) -> R,
              CEILING: Cmp<PRIORITY, Output = Greater> + Cmp<UMAX, Output = Less> + Level
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(<CEILING>::hw());
            barrier!();
            let ret = f(
                Ref::new(&*self.peripheral.get()),
                &C { _marker: PhantomData },
            );
            barrier!();
            basepri::write(old_basepri);
            ret
        }
    }
}

unsafe impl<T, C> Sync for Peripheral<T, C>
where
    C: Ceiling,
{
}

/// A global critical section
///
/// No task can preempt this critical section
pub fn critical<R, F>(f: F) -> R
where
    F: FnOnce(&CMAX) -> R,
{
    let primask = ::cortex_m::register::primask::read();
    ::cortex_m::interrupt::disable();

    let r = f(&C { _marker: PhantomData });

    // If the interrupts were active before our `disable` call, then re-enable
    // them. Otherwise, keep them disabled
    if primask.is_active() {
        ::cortex_m::interrupt::enable();
    }

    r
}

/// Requests the execution of a `task`
pub fn request<T, P>(_task: fn(T, P))
where
    T: Context + Nr,
    P: Priority,
{
    let nvic = unsafe { &*NVIC.get() };

    match () {
        #[cfg(debug_assertions)]
        () => {
            // NOTE(safe) zero sized type
            let task = unsafe { core::ptr::read(0x0 as *const T) };
            // NOTE(safe) atomic read
            assert!(!nvic.is_pending(task),
                    "Task is already in the pending state");
        }
        #[cfg(not(debug_assertions))]
        () => {}
    }

    // NOTE(safe) zero sized type
    let task = unsafe { core::ptr::read(0x0 as *const T) };
    // NOTE(safe) atomic write
    nvic.set_pending(task);
}


/// A type-level ceiling
pub struct C<T> {
    _marker: PhantomData<T>,
}

/// A type-level priority
pub struct P<T> {
    _marker: PhantomData<T>,
}

impl<T> P<T>
where
    T: Level,
{
    #[doc(hidden)]
    pub fn hw() -> u8 {
        T::hw()
    }
}

/// A valid resource ceiling
///
/// DO NOT IMPLEMENT THIS TRAIT YOURSELF
pub unsafe trait Ceiling {}

/// Type-level `>=` operator
///
/// DO NOT IMPLEMENT THIS TRAIT YOURSELF
pub unsafe trait GreaterThanOrEqual<RHS> {}

/// Interrupt hardware level
///
/// DO NOT IMPLEMENT THIS TRAIT YOURSELF
pub unsafe trait Level {
    /// Interrupt hardware level
    fn hw() -> u8;
}

/// A valid task priority
///
/// DO NOT IMPLEMENT THIS TRAIT YOURSELF
pub unsafe trait Priority {}


/// Convert a logical priority to a shifted hardware prio
/// as used by the NVIC and basepri registers
/// Notice, wrapping causes a panic due to u8
pub fn logical2hw(logical: u8) -> u8 {
    ((1 << PRIORITY_BITS) - logical) << (8 - PRIORITY_BITS)
}

/// Convert a shifted hardware prio to a logical priority
/// as used by the NVIC and basepri registers
/// Notice, wrapping causes a panic due to u8
pub fn hw2logical(hw: u8) -> u8 {
    (1 << PRIORITY_BITS) - (hw >> (8 - PRIORITY_BITS))
}


/// Priority 0, the lowest priority
pub type P0 = P<::typenum::U0>;

/// Declares tasks
#[macro_export]
macro_rules! tasks {
    ($krate:ident, {
        $($task:ident: ($Interrupt:ident, $P:ident),)*
    }) => {
        fn main() {
            $crate::critical(|cmax| {
                fn signature(_: fn($crate::P0, &$crate::CMAX)) {}

                signature(init);
                let p0 = unsafe { ::core::ptr::read(0x0 as *const _) };
                init(p0, cmax);
                set_priorities();
                enable_tasks();
            });

            fn signature(_: fn($crate::P0) -> !) {}

            signature(idle);
            let p0 = unsafe { ::core::ptr::read(0x0 as *const _) };
            idle(p0);

            fn set_priorities() {
                // NOTE(safe) this function runs in an interrupt free context
                let _nvic = unsafe { &*$crate::NVIC.get() };

                $(
                    {
                        let hw = $crate::$P::hw();
                        if hw != 0 {
                            unsafe {
                                _nvic.set_priority
                                    (::$krate::interrupt::Interrupt::$Interrupt,
                                     hw,
                                    );
                            }
                        }
                    }
                )*

                // TODO freeze the NVIC.IPR register using the MPU, if available
            }

            fn enable_tasks() {
                // NOTE(safe) this function runs in an interrupt free context
                let _nvic = unsafe { &*$crate::NVIC.get() };

                $(
                    _nvic.enable(::$krate::interrupt::Interrupt::$Interrupt);
                )*
            }

            #[allow(dead_code)]
            fn is_priority<P>()
            where
                P: $crate::Priority,
            {
            }

            #[allow(dead_code)]
            #[link_section = ".rodata.interrupts"]
            #[used]
            static INTERRUPTS: ::$krate::interrupt::Handlers =
                ::$krate::interrupt::Handlers {
                $(
                    $Interrupt: {
                        extern "C" fn $task(
                            task: ::$krate::interrupt::$Interrupt
                        ) {
                            is_priority::<$crate::$P>();
                            ::$task(
                                task, unsafe {
                                    ::core::ptr::read(0x0 as *const $crate::$P)
                                }
                            )
                        }

                        $task
                    },
                )*
                    ..::$krate::interrupt::DEFAULT_HANDLERS
                };
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/prio.rs"));
