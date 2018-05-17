//! The `idle` function

use Priority;

use typenum::consts::U0;

/// The execution context of `idle`
pub struct Context {
    /// The starting priority of `idle`
    pub priority: Priority<U0>,

    /// Resources assigned to `idle`
    pub resources: Resources,
}

/// Resources assigned to `idle`
#[allow(non_snake_case)]
pub struct Resources {
    /// Example of a resource assigned to `idle`
    pub KEY: KEY,
}

#[doc(hidden)]
pub struct KEY;
