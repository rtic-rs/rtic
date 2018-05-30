//! Real Time for The Masses: a high performance and predictable bare metal task scheduler
//!
//! # Features
//!
//! - Priority based scheduler implemented mostly in hardware with minimal bookkeeping and overhead.
//! - Tasks can be started in response to events or scheduled on-demand
//! - Message passing between tasks
//! - Data race free sharing of resources (e.g. memory) between tasks using the Priority Ceiling
//!   Protocol (PCP).
//! - Guaranteed dead lock free execution
//! - Doesn't need a memory allocator to operate
//!
//! # User guide and internal documentation
//!
//! Check [the RTFM book] instead. These auto-generated docs only contain the API reference and some
//! examples.
//!
//! [the RTFM book]: TODO
//!
//! # `app!`
//!
//! The `app!` macro contains the specification of an application. It declares the tasks that
//! compose the application of and how resources (`static` variables) are distributed across them.
//!
//! This section describes the syntax of the `app!` macro.
//!
//! ## `app.device`
//!
//! ``` ignore
//! app! {
//!     device: some::path,
//! }
//! ```
//!
//! This field specifies the target device as a path to a crate generated using [`svd2rust`]
//! v0.13.x.
//!
//! [`svd2rust`]: https://crates.io/crates/svd2rust
//!
//! ## `app.resources`
//!
//! This section contains a list of `static` variables that will be used as resources. These
//! variables don't need to be assigned an initial value. If a resource lacks an initial value it
//! will have to be assigned one in `init`.
//!
//! ``` ignore
//! app! {
//!     resources: {
//!         // Resource with initial value; its initial value is stored in Flash
//!         static STATE: bool = false;
//!
//!         // Resource without initial value; it will be initialized at runtime
//!         static KEY: [u8; 128];
//!
//!         // ..
//!     }
//! }
//! ```
//!
//! ## `app.free_interrupts`
//!
//! This a list of interrupts that the RTFM runtime can use to dispatch on-demand tasks.
//!
//! ## `app.init`
//!
//! This section describes the context of the [`init`][fn@init]ialization function.
//!
//! ``` ignore
//! app! {
//!     init: {
//!         body: some::path,
//!         resources: [A, B],
//!         schedule_now: [on_demand_task],
//!         schedule_after: [task_a],
//!     }
//! }
//! ```
//!
//! ### `app.init.body`
//!
//! This is the path to the `init` function. If omitted this field will default to `init`.
//!
//! ### `app.init.resources`
//!
//! The resources assigned to, and owned by, `init`. This field is optional; if omitted this field
//! defaults to an empty list.
//!
//! ### `app.init.schedule_now` / `app.init.schedule_after`
//!
//! List of tasks `init` can schedule via the [`schedule_now`] and [`schedule_after`] APIs.
//!
//! [`schedule_now`]: trait.ScheduleNow.html
//! [`schedule_after`]: trait.ScheduleAfter.html
//!
//! ## `app.idle`
//!
//! ## `app.tasks`
//!
//! This section contains a list of tasks. These tasks can be event tasks or on-demand tasks.
//!
//! ``` ignore
//! app! {
//!     tasks: {
//!         event_task: {
//!             body: some::path,
//!             interrupt: USART1,
//!             resources: [STATE],
//!             schedule_now: [on_demand_task],
//!             schedule_after: [task_a],
//!         },
//!
//!         on_demand_task: {
//!             body: some::other::path,
//!             instances: 2,
//!             resources: [STATE],
//!             schedule_now: [on_demand_task],
//!             schedule_after: [task_a],
//!         },
//!
//!         // more tasks here
//!     }
//! }
//! ```
//!
//! ### `app.tasks.$.body`
//!
//! The path to the body of the task. This field is optional; if omitted the path defaults to the
//! name of the task.
//!
//! ### `app.tasks.$.interrupt`
//!
//! Event tasks only. This is the event, or interrupt source, that triggers the execution of the
//! task.
//!
//! ### `app.tasks.$.instances`
//!
//! On-demand tasks only. The maximum number of times this task can be scheduled and remain in a
//! pending execution, or ready, state. This field is optional; if omitted, it defaults to `1`.
//!
//! ### `app.tasks.$.resources`
//!
//! The resources assigned to this task. This field is optional; if omitted this field defaults to
//! an empty list.

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
extern crate stable_deref_trait;

use cortex_m::interrupt;
pub use cortex_m_rtfm_macros::app;
use typenum::consts::*;
use typenum::Unsigned;

pub use resource::{Priority, SharedResource, Resource};

#[doc(hidden)]
pub mod _impl;
pub mod event_task;
pub mod idle;
pub mod init;
pub mod on_demand_task;
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

/// The `init`ialization function takes care of system and resource initialization
pub fn init(_ctxt: init::Context) -> init::LateResources {
    unimplemented!()
}

/// When no task is being executed the processor resumes the execution of the `idle` function
pub fn idle(_ctxt: idle::Context) -> ! {
    unimplemented!()
}

/// The `schedule_now` interface
pub trait ScheduleNow {
    /// Optional message sent to the scheduled task
    type Payload;

    /// Schedules a task to run right away
    ///
    /// This method will return an error if the maximum number of `instances` of the task are
    /// already pending execution.
    ///
    /// If `"timer-queue"` is enabled the newly scheduled task will inherit the `baseline` of the
    /// *current* task.
    ///
    /// *NOTE* that the `payload` argument is not required if the task has no input, i.e. its input
    /// type is `()`
    fn schedule_now<P>(
        &mut self,
        priority: &mut Priority<P>,
        payload: Self::Payload,
    ) -> Result<(), Self::Payload>;
}

/// The `schedule_after` interface
///
/// *NOTE* that this API is only available if the `"timer-queue"` feature is enabled.
pub trait ScheduleAfter {
    /// Optional message sent to the scheduled task
    type Payload;

    /// Schedules a task to run `offset` ticks after the *current* task `baseline`.
    ///
    /// This method will return an error if the maximum number of instances of the task are pending
    /// execution.
    ///
    /// *NOTE* that the `payload` argument is not required if the task has no input, i.e. its input
    /// type is `()`
    fn schedule_after<P>(
        &mut self,
        priority: &mut Priority<P>,
        offset: u32,
        payload: Self::Payload,
    ) -> Result<(), Self::Payload>;
}
