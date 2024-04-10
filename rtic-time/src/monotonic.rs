//! Structs and traits surrounding the [`Monotonic`](crate::Monotonic) trait.

pub use timer_queue_based_monotonic::{
    TimerQueueBasedDuration, TimerQueueBasedInstant, TimerQueueBasedMonotonic,
};

mod embedded_hal_macros;
mod timer_queue_based_monotonic;
