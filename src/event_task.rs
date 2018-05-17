//! An event task, a task that starts in response to an event (interrupt source)

use Priority;

/// The execution context of this event task
pub struct Context {
    /// The time at which this task started executing
    ///
    /// *NOTE* that this is not the *arrival* time of the event that started this task. Due to
    /// prioritization of other tasks this task could have started much later than the time the
    /// event arrived at.
    ///
    /// *This field is only available if the `"timer-queue"` feature is enabled*
    pub baseline: u32,

    /// The input of this task
    pub input: Input,

    /// The starting priority of this task
    pub priority: Priority<P>,

    /// Resources assigned to this event task
    pub resources: Resources,

    /// Tasks that this event task can schedule
    pub tasks: Tasks,
}

#[doc(hidden)]
pub struct Input;

#[doc(hidden)]
pub struct P;

/// Resources assigned to this event task
pub struct Resources {}

/// Tasks that this event task can schedule
pub struct Tasks {}
