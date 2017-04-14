#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
extern crate typenum;

use core::cell::UnsafeCell;
use core::marker::PhantomData;
#[cfg(not(thumbv6m))]
use core::ops::Sub;

use cortex_m::ctxt::Context;
use cortex_m::interrupt::Nr;
#[cfg(not(thumbv6m))]
use cortex_m::register::{basepri, basepri_max};
use typenum::{Cmp, Equal, Unsigned};
#[cfg(not(thumbv6m))]
use typenum::{B1, Greater, Less, Sub1};

pub use cortex_m::ctxt::Local;
pub use cortex_m::asm::wfi;

#[doc(hidden)]
pub use cortex_m::peripheral::NVIC;
#[doc(hidden)]
pub use cortex_m::interrupt::free;

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

impl<T, CEILING> Resource<T, C<CEILING>>
where
    C<CEILING>: Ceiling,
{
    /// Borrows the resource for the duration of another resource's critical
    /// section
    ///
    /// This operation is zero cost and doesn't impose any additional blocking
    pub fn borrow<'cs, SCEILING>(
        &'static self,
        _system_ceiling: &'cs C<SCEILING>,
    ) -> &'cs T
    where
        SCEILING: GreaterThanOrEqual<CEILING>,
    {
        unsafe { &*self.data.get() }
    }

    /// Claims the resource at the task with highest priority
    ///
    /// This operation is zero cost and doesn't impose any additional blocking
    pub fn claim<'task, PRIORITY>(
        &'static self,
        _priority: &'task P<PRIORITY>,
    ) -> &'task T
    where
        CEILING: Cmp<PRIORITY, Output = Equal>,
        P<PRIORITY>: Priority,
    {
        unsafe { &*self.data.get() }
    }

    /// Like [Resource.claim](struct.Resource.html#method.claim) but returns a
    /// `&mut-` reference
    pub fn claim_mut<'task, PRIORITY>(
        &'static self,
        _priority: &'task mut P<PRIORITY>,
    ) -> &'task mut T
    where
        CEILING: Cmp<PRIORITY, Output = Equal>,
        P<PRIORITY>: Priority,
    {
        unsafe { &mut *self.data.get() }
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
    pub fn lock<R, PRIORITY, F>(
        &'static self,
        _priority: &P<PRIORITY>,
        f: F,
    ) -> R
    where
        F: FnOnce(&T, C<CEILING>) -> R,
        C<CEILING>: Ceiling,
        CEILING: Cmp<PRIORITY, Output = Greater> + Cmp<UMAX, Output = Less>
            + Level,
        P<PRIORITY>: Priority,
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(<CEILING>::hw());
            barrier!();
            let ret = f(
                &*self.data.get(),
                C {
                    _0: (),
                    _marker: PhantomData,
                },
            );
            barrier!();
            basepri::write(old_basepri);
            ret
        }
    }

    /// Like [Resource.lock](struct.Resource.html#method.lock) but returns a
    /// `&mut-` reference
    ///
    /// This method has additional an additional constraint: you can't borrow a
    /// resource that has ceiling equal `CEILING`. This constraint is required
    /// to preserve Rust aliasing rules.
    #[cfg(not(thumbv6m))]
    pub fn lock_mut<R, PRIORITY, F>(
        &'static self,
        _priority: &mut P<PRIORITY>,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut T, C<Sub1<CEILING>>) -> R,
        C<CEILING>: Ceiling,
        CEILING: Sub<B1>,
        CEILING: Cmp<PRIORITY, Output = Greater> + Cmp<UMAX, Output = Less>
            + Level,
        P<PRIORITY>: Priority,
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(<CEILING>::hw());
            barrier!();
            let ret = f(
                &mut *self.data.get(),
                C {
                    _0: (),
                    _marker: PhantomData,
                },
            );
            barrier!();
            basepri::write(old_basepri);
            ret
        }
    }
}

unsafe impl<T, CEILING> Sync for Resource<T, CEILING>
where
    CEILING: Ceiling,
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

impl<Periph, CEILING> Peripheral<Periph, C<CEILING>>
where
    C<CEILING>: Ceiling,
{
    /// See [Resource.borrow](./struct.Resource.html#method.borrow)
    pub fn borrow<'cs, SCEILING>(
        &'static self,
        _system_ceiling: &'cs C<SCEILING>,
    ) -> &'cs Periph
    where
        SCEILING: GreaterThanOrEqual<CEILING>,
    {
        unsafe { &*self.peripheral.get() }
    }

    /// See [Resource.claim](./struct.Resource.html#method.claim)
    pub fn claim<'task, PRIORITY>(
        &'static self,
        _priority: &'task P<PRIORITY>,
    ) -> &'task Periph
    where
        CEILING: Cmp<PRIORITY, Output = Equal>,
        P<PRIORITY>: Priority,
    {
        unsafe { &*self.peripheral.get() }
    }

    /// See [Resource.lock](./struct.Resource.html#method.lock)
    #[cfg(not(thumbv6m))]
    pub fn lock<R, PRIORITY, F>(
        &'static self,
        _priority: &P<PRIORITY>,
        f: F,
    ) -> R
    where
        F: FnOnce(&Periph, C<CEILING>) -> R,
        C<CEILING>: Ceiling,
        CEILING: Cmp<PRIORITY, Output = Greater> + Cmp<UMAX, Output = Less>
            + Level,
        P<PRIORITY>: Priority,
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(<CEILING>::hw());
            barrier!();
            let ret = f(
                &*self.peripheral.get(),
                C {
                    _0: (),
                    _marker: PhantomData,
                },
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

/// Requests the execution of the task `task`
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
    _0: (),
    _marker: PhantomData<T>,
}

/// A type-level priority
pub struct P<T> {
    _0: (),
    _marker: PhantomData<T>,
}

impl<T> P<T>
where
    T: Level,
{
    pub fn hw() -> u8 {
        T::hw()
    }
}

/// A valid ceiling
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
    fn hw() -> u8;
}

/// A valid priority level
///
/// DO NOT IMPLEMENT THIS TRAIT YOURSELF
pub unsafe trait Priority {}

fn logical2hw(logical: u8) -> u8 {
    ((1 << PRIORITY_BITS) - logical) << (8 - PRIORITY_BITS)
}

/// Priority 0, the lowest priority
pub type P0 = P<::typenum::U0>;

unsafe impl Priority for P0 {}

/// Declares tasks
#[macro_export]
macro_rules! tasks {
    ($krate:ident, {
        $($task:ident: ($Interrupt:ident, $P:ident),)*
    }) => {
        fn main() {
            $crate::free(|_| {
                init(unsafe { ::core::ptr::read(0x0 as *const $crate::CMAX )});
                set_priorities();
                enable_tasks();
            });

            idle(unsafe { ::core::ptr::read(0x0 as *const P0) });

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
