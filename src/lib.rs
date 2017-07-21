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
//! In increasing grade of complexity: [examples](./examples/index.html)

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(optin_builtin_traits)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm_macros;
extern crate static_ref;

use core::cell::UnsafeCell;

pub use cortex_m_rtfm_macros::app;
pub use cortex_m::asm::{bkpt, wfi};
pub use cortex_m::interrupt::CriticalSection;
pub use cortex_m::interrupt::free as atomic;
pub use static_ref::Static;
use cortex_m::interrupt::Nr;
#[cfg(not(armv6m))]
use cortex_m::register::{basepri, basepri_max};

pub mod examples;

/// A resource, a means to share data between tasks
pub trait Resource {
    /// The data protected by the resource
    type Data;

    /// Borrows the resource data for the duration of a *global* critical
    /// section
    fn borrow<'cs>(
        &'cs self,
        cs: &'cs CriticalSection,
    ) -> &'cs Static<Self::Data>;

    /// Mutable variant of `borrow`
    fn borrow_mut<'cs>(
        &'cs mut self,
        cs: &'cs CriticalSection,
    ) -> &'cs mut Static<Self::Data>;

    /// Claims the resource data for the span of the closure `f`. For the
    /// duration of the closure other tasks that may access the resource data
    /// are prevented from preempting the current task.
    fn claim<R, F>(&self, t: &mut Threshold, f: F) -> R
    where
        F: FnOnce(&Static<Self::Data>, &mut Threshold) -> R;

    /// Mutable variant of `claim`
    fn claim_mut<R, F>(&mut self, t: &mut Threshold, f: F) -> R
    where
        F: FnOnce(&mut Static<Self::Data>, &mut Threshold) -> R;
}

impl<T> Resource for Static<T> {
    type Data = T;

    fn borrow<'cs>(&'cs self, _cs: &'cs CriticalSection) -> &'cs Static<T> {
        self
    }

    fn borrow_mut<'cs>(
        &'cs mut self,
        _cs: &'cs CriticalSection,
    ) -> &'cs mut Static<T> {
        self
    }

    fn claim<R, F>(&self, t: &mut Threshold, f: F) -> R
    where
        F: FnOnce(&Static<Self::Data>, &mut Threshold) -> R,
    {
        f(self, t)
    }

    fn claim_mut<R, F>(&mut self, t: &mut Threshold, f: F) -> R
    where
        F: FnOnce(&mut Static<Self::Data>, &mut Threshold) -> R,
    {
        f(self, t)
    }
}

#[doc(hidden)]
pub unsafe fn claim<T, U, R, F, G>(
    data: *mut T,
    ceiling: u8,
    nvic_prio_bits: u8,
    t: &mut Threshold,
    f: F,
    g: G,
) -> R
where
    F: FnOnce(U, &mut Threshold) -> R,
    G: FnOnce(*mut T) -> U,
{
    let max_priority = 1 << nvic_prio_bits;
    if ceiling > t.value {
        match () {
            #[cfg(armv6m)]
            () => {
                atomic(|_| f(g(data), &mut Threshold::new(max_priority)))
            }
            #[cfg(not(armv6m))]
            () => {
                if ceiling == max_priority {
                    atomic(|_| f(g(data), &mut Threshold::new(max_priority)))
                } else {
                    let old = basepri::read();
                    let hw = (max_priority - ceiling) << (8 - nvic_prio_bits);
                    basepri_max::write(hw);
                    let ret = f(g(data), &mut Threshold::new(ceiling));
                    basepri::write(old);
                    ret
                }
            }
        }
    } else {
        f(g(data), t)
    }
}

#[doc(hidden)]
pub struct Cell<T> {
    data: UnsafeCell<T>,
}

#[doc(hidden)]
impl<T> Cell<T> {
    pub const fn new(data: T) -> Self {
        Cell {
            data: UnsafeCell::new(data),
        }
    }

    pub fn get(&self) -> *mut T {
        self.data.get()
    }
}

unsafe impl<T> Sync for Cell<T>
where
    T: Send,
{
}

/// Preemption threshold token
///
/// The preemption threshold indicates the priority a task must have to preempt
/// the current context. For example a threshold of 2 indicates that only
/// interrupts / exceptions with a priority of 3 or greater can preempt the
/// current context
pub struct Threshold {
    value: u8,
}

impl Threshold {
    #[doc(hidden)]
    pub unsafe fn new(value: u8) -> Self {
        Threshold { value }
    }
}

impl !Send for Threshold {}

/// Sets an interrupt as pending
pub fn set_pending<I>(interrupt: I)
where
    I: Nr,
{
    // NOTE(safe) atomic write
    let nvic = unsafe { &*cortex_m::peripheral::NVIC.get() };
    nvic.set_pending(interrupt);
}

/// Binds a task `$handler` to the interrupt / exception `$NAME`
#[macro_export]
macro_rules! task {
    ($NAME:ident, $handler:path) => {
        #[allow(non_snake_case)]
        #[allow(unsafe_code)]
        #[no_mangle]
        pub unsafe extern "C" fn $NAME() {
            let f: fn(&mut $crate::Threshold, ::$NAME::Resources) = $handler;

            f(
                &mut $crate::Threshold::new(::$NAME::$NAME),
                ::$NAME::Resources::new(),
            );
        }
    };

    ($NAME:ident, $handler:path, $locals:ident {
        $(static $var:ident: $ty:ty = $expr:expr;)+
    }) => {
        #[allow(non_snake_case)]
        struct $locals {
            $($var: $crate::Static<$ty>,)+
        }

        #[allow(non_snake_case)]
        #[allow(unsafe_code)]
        #[no_mangle]
        pub unsafe extern "C" fn $NAME() {
            let f: fn(
                &mut $crate::Threshold,
                &mut $locals,
                ::$NAME::Resources,
            ) = $handler;

            static mut LOCALS: $locals = $locals {
                $($var: unsafe { $crate::Static::new($expr) },)+
            };

            f(
                &mut $crate::Threshold::new(::$NAME::$NAME),
                &mut LOCALS,
                ::$NAME::Resources::new(),
            );
        }
    };
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
pub enum Exception {
    /// System service call via SWI instruction
    SVCALL,
    /// Pendable request for system service
    PENDSV,
    /// System tick timer
    SYS_TICK,
}

impl Exception {
    #[doc(hidden)]
    pub fn nr(&self) -> usize {
        match *self {
            Exception::SVCALL => 11,
            Exception::PENDSV => 14,
            Exception::SYS_TICK => 15,
        }
    }
}
