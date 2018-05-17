//! An schedulable task

use Priority;

/// The execution context of this schedulable task
pub struct Context {
    /// The time at which this task was scheduled to run
    ///
    /// *NOTE* that this is not the *start* time of the task. Due to scheduling overhead a task will
    /// always start a bit later than its scheduled time. Also due to prioritization of other tasks
    /// a task may start much later than its scheduled time.
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
