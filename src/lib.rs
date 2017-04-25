//! RTFM: Real Time For the Masses (ARM Cortex-M edition)
//!
//! RTFM is a framework for building event-driven applications for ARM Cortex-M
//! microcontrollers.
//!
//! This crate is based on the RTFM framework created by [prof. Per
//! Lindgren][per] and uses a simplified version of the Stack Resource Policy as
//! scheduling policy. Check the [references] for details.
//!
//! [per]: https://www.ltu.se/staff/p/pln-1.11258?l=en
//! [references]: ./index.html#references
//!
//! # Features
//!
//! - **Event triggered tasks** as the unit of concurrency.
//! - Supports prioritization of tasks and, thus, **preemptive multitasking**.
//! - **Data race free memory sharing** through fine grained *partial* critical
//!   sections.
//! - **Deadlock free execution** guaranteed at compile time.
//! - **Minimal overhead** as the scheduler has no software component / runtime; the
//!   hardware does all the scheduling.
//! - Full support for all Cortex M3, M4 and M7 devices. M0(+) is also supported
//!   but the whole API is not available (due to missing hardware features).
//! - The number of task priority levels is configurable at compile time through
//!   the `P2` (4 levels), `P3` (8 levels), etc. Cargo features. The number of
//!   priority levels supported by the hardware is device specific but this
//!   crate defaults to 16 as that's the most common scenario.
//! - This task model is amenable to known WCET (Worst Case Execution Time)
//!   analysis and scheduling analysis techniques. (Though we haven't yet
//!   developed Rust friendly tooling for that.)
//!
//! # Limitations
//!
//! - Task priorities must remain constant at runtime.
//!
//! # Dependencies
//!
//! - A device crate generated using [`svd2rust`] v0.6.x
//! - A `start` lang time: Vanilla `main` must be supported in binary crates.
//!   You can use the [`cortex-m-rt`] crate to fulfill the requirement
//!
//! [`svd2rust`]: https://docs.rs/svd2rust/0.6.1/svd2rust/
//! [`cortex-m-rt`]: https://docs.rs/cortex-m-rt/0.1.1/cortex_m_rt/
//!
//! # Examples
//!
//! Ordered in increasing level of complexity:
//!
//! - [Zero tasks](./index.html#zero-tasks)
//! - [One task](./index.html#one-task)
//! - [Two "serial" tasks](./index.html#two-serial-tasks)
//! - [Preemptive multitasking](./index.html#preemptive-multitasking)
//! - [Peripherals as resources](./index.html#peripherals-as-resources)
//!
//! ## Zero tasks
//!
//! ``` ignore
//! #![no_std]
//!
//! #[macro_use] // for the `hprintln!` macro
//! extern crate cortex_m;
//!
//! // before main initialization + `start` lang item
//! extern crate cortex_m_rt;
//!
//! #[macro_use] // for the `tasks!` macro
//! extern crate cortex_m_rtfm as rtfm;
//!
//! // device crate generated using svd2rust
//! extern crate stm32f30x;
//!
//! use rtfm::{C16, P0};
//!
//! // TASKS (None in this example)
//! tasks!(stm32f30x, {});
//!
//! // INITIALIZATION PHASE
//! fn init(_priority: P0, _ceiling: &C16) {
//!     hprintln!("INIT");
//! }
//!
//! // IDLE LOOP
//! fn idle(_priority: P0) -> ! {
//!     hprintln!("IDLE");
//!
//!     // Sleep
//!     loop {
//!         rtfm::wfi();
//!     }
//! }
//! ```
//!
//! Expected output:
//!
//! ``` text
//! INIT
//! IDLE
//! ```
//!
//! The `tasks!` macro overrides the `main` function and imposes the following
//! structure into your program:
//!
//! - `init`, the initialization phase, runs first. This function is executed
//!   inside a *global* critical section and can't be preempted.
//!
//! - `idle`, a never ending function that runs after `init`.
//!
//! Note that both `init` and `idle` have priority 0, the lowest priority.
//!
//! # One task
//!
//! ``` ignore
//! #![no_std]
//!
//! extern crate cortex_m_rt;
//! #[macro_use]
//! extern crate cortex_m_rtfm as rtfm;
//! extern crate stm32f30x;
//!
//! use stm32f30x::interrupt::Tim7;
//! use rtfm::{C16, Local, P0, P1};
//!
//! // INITIALIZATION PHASE
//! fn init(_priority: P0, _ceiling: &C16) {
//!     // Configure TIM7 for periodic interrupts
//!     // Configure GPIO for LED driving
//! }
//!
//! // IDLE LOOP
//! fn idle(_priority: P0) -> ! {
//!     // Sleep
//!     loop { rtfm::wfi() }
//! }
//!
//! // TASKS
//! tasks!(stm32f30x, {
//!     periodic: (Tim7, P1, true),
//! });
//!
//! fn periodic(mut task: Tim7, _priority: P1) {
//!     // Task local data
//!     static STATE: Local<bool, Tim7> = Local::new(false);
//!
//!     let state = STATE.borrow_mut(&mut task);
//!
//!     // Toggle state
//!     *state = !*state;
//!
//!     // Blink an LED
//!     if *state {
//!         LED.on();
//!     } else {
//!         LED.off();
//!     }
//! }
//! ```
//!
//! Here we define a task named `periodic` and bind it to the `Tim7`
//! interrupt. The `periodic` task will run every time the `Tim7` interrupt
//! is triggered. We assign to this task a priority of 1 (`P1`); this is the
//! lowest priority that a task can have.
//!
//! We use the [`Local`](./struct.Local.html) abstraction to add state to the
//! task; this task local data will be preserved across runs of the `periodic`
//! task. Note that `STATE` is owned by the `periodic` task, in the sense that
//! no other task can access it; this is reflected in its type signature (the
//! `Tim7` type parameter).
//!
//! # Two "serial" tasks
//!
//! ``` ignore
//! #![no_std]
//!
//! extern crate cortex_m_rt;
//! #[macro_use]
//! extern crate cortex_m_rtfm as rtfm;
//! extern crate stm32f30x;
//!
//! use core::cell::Cell;
//!
//! use stm32f30x::interrupt::{Tim6Dacunder, Tim7};
//! use rtfm::{C1, C16, P0, P1, Resource};
//!
//! // omitted: `idle`, `init`
//!
//! tasks!(stm32f30x, {
//!     t1: (Tim6Dacunder, P1, true),
//!     t2: (Tim7, P1, true),
//! });
//!
//! // Data shared between tasks `t1` and `t2`
//! static COUNTER: Resource<Cell<u32>, C1> = Resource::new(Cell::new(0));
//!
//! fn t1(_task: Tim6Dacunder, priority: P1) {
//!     let ceiling = priority.as_ceiling();
//!
//!     let counter = COUNTER.access(&priority, ceiling);
//!
//!     counter.set(counter.get() + 1);
//! }
//!
//! fn t2(_task: Tim7, priority: P1) {
//!     let ceiling = priority.as_ceiling();
//!
//!     let counter = COUNTER.access(&priority, ceiling);
//!
//!     counter.set(counter.get() + 2);
//! }
//! ```
//!
//! Here we declare two tasks, `t1` and `t2`; both with a priority of 1 (`P1`).
//! As both tasks have the same priority, we say that they are *serial* tasks in
//! the sense that `t1` can only run *after* `t2` is done and vice versa; i.e.
//! there's no preemption.
//!
//! To share data between these two tasks, we use the
//! [`Resource`](./struct.Resource.html) abstraction. As the tasks can't preempt
//! each other, they can access the `COUNTER` resource using the zero cost
//! [`access`](./struct.Resource.html#method.access) method -- no
//! synchronization needed.
//!
//! `COUNTER` has an extra type parameter: `C1`. This is the *ceiling* of the
//! resource. For now suffices to say that the ceiling must be the maximum of
//! the priorities of all the tasks that access the resource -- in this case,
//! `C1 == max(P1, P1)`. If you try a smaller value like `C0`, you'll find out
//! that your program doesn't compile.
//!
//! # Preemptive multitasking
//!
//! ``` ignore
//! #![no_std]
//!
//! extern crate cortex_m_rt;
//! #[macro_use]
//! extern crate cortex_m_rtfm as rtfm;
//! extern crate stm32f30x;
//!
//! use core::cell::Cell;
//!
//! use stm32f30x::interrupt::{Tim6Dacunder, Tim7};
//! use rtfm::{C2, C16, P0, P1, P2, Resource};
//!
//! // omitted: `idle`, `init`
//!
//! tasks!(stm32f30x, {
//!     t1: (Tim6Dacunder, P1, true),
//!     t2: (Tim7, P2, true),
//! });
//!
//! static COUNTER: Resource<Cell<u32>, C2> = Resource::new(Cell::new(0));
//!
//! fn t1(_task: Tim6Dacunder, priority: P1) {
//!     // ..
//!
//!    let ceiling: &C1 = priority.as_ceiling();
//!
//!    ceiling.raise(&COUNTER, |ceiling: &C2| {
//!        let counter = COUNTER.access(&priority, ceiling);
//!
//!        counter.set(counter.get() + 1);
//!    });
//!
//!     // ..
//! }
//!
//! fn t2(_task: Tim7, priority: P2) {
//!     let ceiling = priority.as_ceiling();
//!
//!     let counter = COUNTER.access(&priority, ceiling);
//!
//!     counter.set(counter.get() + 2);
//! }
//! ```
//!
//! Now we have a variation of the previous example. Like before, `t1` has a
//! priority of 1 (`P1`) but `t2` now has a priority of 2 (`P2`). This means
//! that `t2` can preempt `t1` if a `Tim7` interrupt occurs while `t1` is
//! being executed.
//!
//! To avoid data races, `t1` must modify `COUNTER` in an atomic way; i.e. `t2`
//! most not preempt `t1` while `COUNTER` is being modified. This is
//! accomplished by [`raise`](./struct.C.html#method.raise)-ing the `ceiling`.
//! This creates a critical section, denoted by a closure, for whose span
//! `COUNTER` is accessible but `t2` is blocked from preempting `t1`.
//!
//! How `t2` accesses `COUNTER` remains unchanged. Since `t1` can't preempt `t2`
//! due to the differences in priority; no critical section is needed in `t2`.
//!
//! Note that the ceiling of `COUNTER` had to change to `C2`. This is required
//! because the ceiling must be the maximum between `P1` and `P2`.
//!
//! Finally, it should be noted that the critical section in `t1` will only
//! block tasks with a priority of 2 or lower. This is exactly what the ceiling
//! represents: it's the "bar" that a task priority must pass in order to be
//! able to preempt the current task / critical section.
//!
//! # Peripherals as resources
//!
//! ``` ignore
//! #![no_std]
//!
//! extern crate cortex_m_rt;
//! #[macro_use]
//! extern crate cortex_m_rtfm as rtfm;
//! extern crate stm32f100xx;
//!
//! use rtfm::{C0, C16, P0, Peripheral};
//!
//! static GPIOA: Peripheral<stm32f100xx::Gpioa, C0> =
//!     unsafe { Peripheral::new(stm32f100xx::GPIOA) };
//!
//! static RCC: Peripheral<stm32f100xx::Rcc, C0> =
//!     unsafe { Peripheral::new(stm32f100xx::RCC) };
//!
//! tasks!(stm32f100xx, {});
//!
//! fn init(priority: P0, ceiling: &C16) {
//!     let gpioa = GPIOA.access(&priority, &ceiling);
//!     let rcc = RCC.access(&priority, &ceiling);
//!
//!     // ..
//! }
//!
//! fn idle(_priority: P0) -> ! {
//!     // Sleep
//!     loop { rtfm::wfi() }
//! }
//! ```
//!
//! Peripherals are global resources too and as such they can be protected in
//! the same way as `Resource`s using the
//! [`Peripheral`](./struct.Peripheral.html) abstraction.
//!
//! `Peripheral` and `Resource` has pretty much the same API except that
//! `Peripheral.new` is `unsafe`. Care must be taken to NOT alias peripherals;
//! i.e. don't create two `Peripheral`s that point to the same register block.
//!
//! # References
//!
//! - Baker, T. P. (1991). Stack-based scheduling of realtime processes.
//!   *Real-Time Systems*, 3(1), 67-99.
//!
//! > The original Stack Resource Policy paper. [PDF].
//!
//! [PDF]: http://www.cs.fsu.edu/~baker/papers/mstacks3.pdf
//!
//! - Eriksson, J., Häggström, F., Aittamaa, S., Kruglyak, A., & Lindgren, P.
//!   (2013, June). Real-time for the masses, step 1: Programming API and static
//!   priority SRP kernel primitives. In Industrial Embedded Systems (SIES),
//!   2013 8th IEEE International Symposium on (pp. 110-113). IEEE.
//!
//! > A description of the RTFM task and resource model. [PDF]
//!
//! [PDF]: http://www.diva-portal.org/smash/get/diva2:1005680/FULLTEXT01.pdf

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
extern crate static_ref;
extern crate typenum;

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr;

use cortex_m::ctxt::Context;
use cortex_m::interrupt::Nr;
#[cfg(not(thumbv6m))]
use cortex_m::register::{basepri, basepri_max};
use static_ref::Ref;
use typenum::{Cmp, Greater, U0, Unsigned};
#[cfg(not(thumbv6m))]
use typenum::Less;

pub use cortex_m::asm::{bkpt, wfi};

#[doc(hidden)]
pub use cortex_m::peripheral::NVIC as _NVIC;

/// Compiler barrier
macro_rules! barrier {
    () => {
        asm!(""
             :
             :
             : "memory"
             : "volatile");
    }
}

/// Task local data
///
/// This data can only be accessed by the task `T`
pub struct Local<D, T> {
    _task: PhantomData<T>,
    data: UnsafeCell<D>,
}

impl<T, TASK> Local<T, TASK> {
    /// Creates a task local variable with some initial `value`
    pub const fn new(value: T) -> Self {
        Local {
            _task: PhantomData,
            data: UnsafeCell::new(value),
        }
    }

    /// Borrows the task local data for the duration of the task
    pub fn borrow<'task>(&'static self, _task: &'task TASK) -> &'task T {
        unsafe { &*self.data.get() }
    }

    /// Mutably borrows the task local data for the duration of the task
    pub fn borrow_mut<'task>(
        &'static self,
        _task: &'task mut TASK,
    ) -> &'task mut T {
        unsafe { &mut *self.data.get() }
    }
}

unsafe impl<T, TASK> Sync for Local<T, TASK> {}

/// A resource with ceiling `C`
pub struct Resource<T, C> {
    _ceiling: PhantomData<C>,
    data: UnsafeCell<T>,
}

impl<T, RC> Resource<T, C<RC>>
where
    RC: GreaterThanOrEqual<U0>,
    RC: LessThanOrEqual<UMAX>,
{
    /// Creates a new resource
    pub const fn new(data: T) -> Self {
        Resource {
            _ceiling: PhantomData,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T, RC> Resource<T, C<RC>> {
    /// Grants data race free and deadlock free access to the resource data
    ///
    /// This operation is zero cost and doesn't impose any additional blocking.
    ///
    /// # Requirements
    ///
    /// To access the resource data these conditions must be met:
    ///
    /// - The resource ceiling must be greater than or equal to the task
    ///   priority
    /// - The system ceiling must be greater than or equal to the resource
    ///   ceiling
    pub fn access<'cs, TP, SC>(
        &'static self,
        _priority: &P<TP>,
        _current_ceiling: &'cs C<SC>,
    ) -> Ref<'cs, T>
    where
        RC: GreaterThanOrEqual<TP>,
        SC: GreaterThanOrEqual<RC>,
    {
        unsafe { Ref::new(&*self.data.get()) }
    }
}

unsafe impl<T, C> Sync for Resource<T, C> {}

/// A hardware peripheral as a resource
pub struct Peripheral<P, CEILING>
where
    P: 'static,
{
    peripheral: cortex_m::peripheral::Peripheral<P>,
    _ceiling: PhantomData<CEILING>,
}

impl<P, CEILING> Peripheral<P, C<CEILING>>
where
    CEILING: GreaterThanOrEqual<U0>,
    CEILING: LessThanOrEqual<UMAX>,
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

impl<Periph, RC> Peripheral<Periph, C<RC>> {
    /// See [Resource.access](./struct.Resource.html#method.access)
    pub fn access<'cs, TP, SC>(
        &'static self,
        _priority: &P<TP>,
        _system_ceiling: &'cs C<SC>,
    ) -> Ref<'cs, Periph>
    where
        RC: GreaterThanOrEqual<TP>,
        SC: GreaterThanOrEqual<RC>,
    {
        unsafe { Ref::new(&*self.peripheral.get()) }
    }
}

unsafe impl<T, C> Sync for Peripheral<T, C> {}

/// Runs the closure `f` in a *global* critical section
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

/// Disables the `task`
///
/// The task won't run even if the underlying interrupt is raised
pub fn disable<T, TP>(_task: fn(T, P<TP>))
    where
    T: Context + Nr,
{
    // NOTE(safe) zero sized type
    let _task = unsafe { ptr::read(0x0 as *const T) };

    // NOTE(safe) atomic write
    unsafe { (*_NVIC.get()).disable(_task) }
}

/// Enables the `task`
pub fn enable<T, TP>(_task: fn(T, P<TP>))
where
    T: Context + Nr,
{
    // NOTE(safe) zero sized type
    let _task = unsafe { ptr::read(0x0 as *const T) };

    // NOTE(safe) atomic write
    unsafe { (*_NVIC.get()).enable(_task) }
}

/// Converts a shifted hardware priority into a logical priority
pub fn hw2logical(hw: u8) -> u8 {
    (1 << PRIORITY_BITS) - (hw >> (8 - PRIORITY_BITS))
}

/// Converts a logical priority into a shifted hardware priority, as used by the
/// NVIC and the BASEPRI register
///
/// # Panics
///
/// This function panics if `logical` is outside the closed range
/// `[1, 1 << PRIORITY_BITS]`. Where `PRIORITY_BITS` is the number of priority
/// bits used by the device specific NVIC implementation.
pub fn logical2hw(logical: u8) -> u8 {
    assert!(logical >= 1 && logical <= (1 << PRIORITY_BITS));

    ((1 << PRIORITY_BITS) - logical) << (8 - PRIORITY_BITS)
}

/// Requests the execution of a `task`
pub fn request<T, TP>(_task: fn(T, P<TP>))
where
    T: Context + Nr,
{
    let nvic = unsafe { &*_NVIC.get() };

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

#[doc(hidden)]
pub fn _validate_priority<TP>(_: &P<TP>)
where
    TP: Cmp<U0, Output = Greater> + LessThanOrEqual<UMAX>,
{
}

/// A type-level ceiling
pub struct C<T> {
    _marker: PhantomData<T>,
}

impl<SC> C<SC> {
    /// Raises the system ceiling to match the `resource` ceiling
    pub fn raise<RC, RES, R, F>(&self, _resource: &'static RES, f: F) -> R
    where
        RES: ResourceLike<Ceiling = RC>,
        RC: Cmp<SC, Output = Greater> + Cmp<UMAX, Output = Less> + Unsigned,
        F: FnOnce(&C<RC>) -> R,
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(logical2hw(RC::to_u8()));
            barrier!();
            let ret = f(&C { _marker: PhantomData });
            barrier!();
            basepri::write(old_basepri);
            ret
        }
    }
}

/// A type-level priority
pub struct P<T> {
    _marker: PhantomData<T>,
}

impl<T> P<T>
where
    T: Unsigned,
{
    #[doc(hidden)]
    pub fn _hw() -> u8 {
        logical2hw(T::to_u8())
    }
}

/// Maps a `Resource` / `Peripheral` to its ceiling
///
/// Do not implement this trait yourself. This is an implementation detail.
pub unsafe trait ResourceLike {
    /// The ceiling of the resource
    type Ceiling;
}

unsafe impl<P, RC> ResourceLike for Peripheral<P, C<RC>> {
    type Ceiling = RC;
}

unsafe impl<T, RC> ResourceLike for Resource<T, C<RC>> {
    type Ceiling = RC;
}

/// Type-level `>=` operator
///
/// Do not implement this trait yourself. This is an implementation detail.
pub unsafe trait GreaterThanOrEqual<RHS> {}

/// Type-level `<=` operator
///
/// Do not implement this trait yourself. This is an implementation detail.
pub unsafe trait LessThanOrEqual<RHS> {}

/// A macro to declare tasks
///
/// **NOTE** This macro will expand to a `main` function.
///
/// Each `$task` is bound to an `$Interrupt` handler and has a priority `$P`.
/// `$enabled` indicates whether the task will be enabled before `idle` runs.
///
/// The `$Interrupt` handlers are defined in the `$device` crate.
///
/// Apart from defining the listed `$tasks`, the `init` and `idle` functions
/// must be defined as well. `init` has type signature `fn(P0, &C16)`, and
/// `idle` has signature `fn(P0) -> !`.
#[macro_export]
macro_rules! tasks {
    ($device:ident, {
        $($task:ident: ($Interrupt:ident, $P:ident, $enabled:expr),)*
    }) => {
        fn main() {
            $crate::critical(|cmax| {
                fn validate_signature(_: fn($crate::P0, &$crate::CMAX)) {}

                validate_signature(init);
                let p0 = unsafe { ::core::mem::transmute::<_, P0>(()) };
                init(p0, cmax);
                set_priorities();
                enable_tasks();
            });

            fn validate_signature(_: fn($crate::P0) -> !) {}

            validate_signature(idle);
            let p0 = unsafe { ::core::mem::transmute::<_, P0>(()) };
            idle(p0);

            fn set_priorities() {
                // NOTE(safe) this function runs in an interrupt free context
                let _nvic = unsafe { &*$crate::_NVIC.get() };

                $(
                    {
                        let hw = $crate::$P::_hw();
                        unsafe {
                            _nvic.set_priority(
                                ::$device::interrupt::Interrupt::$Interrupt,
                                hw,
                            );
                        }
                    }
                )*

                // TODO freeze the NVIC.IPR register using the MPU, if available
            }

            fn enable_tasks() {
                // NOTE(safe) this function runs in an interrupt free context
                let _nvic = unsafe { &*$crate::_NVIC.get() };

                $(
                    if $enabled {
                        $crate::enable(::$task);
                    }
                )*
            }

            #[allow(dead_code)]
            #[link_section = ".rodata.interrupts"]
            #[used]
            static INTERRUPTS: ::$device::interrupt::Handlers =
                ::$device::interrupt::Handlers {
                $(
                    $Interrupt: {
                        extern "C" fn $task(
                            task: ::$device::interrupt::$Interrupt
                        ) {
                            let p = unsafe {
                                ::core::mem::transmute::<_, $crate::$P>(())
                            };
                            $crate::_validate_priority(&p);
                            ::$task(task, p)
                        }

                        $task
                    },
                )*
                    ..::$device::interrupt::DEFAULT_HANDLERS
                };
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/prio.rs"));
