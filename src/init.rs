//! The `init`ialization function
use typenum::consts::U255;

use Priority;
pub use _impl::Peripherals as Core;

/// Execution context of `init`
pub struct Context<'a> {
    /// Core (Cortex-M) peripherals
    pub core: Core<'a>,
    /// Device specific peripherals
    pub device: Device,
    /// The priority of `init`
    pub priority: Priority<U255>,
    /// Resources assigned to `init`
    pub resources: Resources,
    /// Tasks that `init` can schedule
    pub tasks: Tasks,
}

/// Device specific peripherals
///
/// The contents of this `struct` will depend on the selected `device`
#[allow(non_snake_case)]
pub struct Device {
    /// Example
    pub GPIOA: GPIOA,
    /// Example
    pub TIM2: TIM2,
    /// Example
    pub USART1: USART1,
    _more: (),
}

#[doc(hidden)]
pub struct GPIOA;

#[doc(hidden)]
pub struct RCC;

#[doc(hidden)]
pub struct TIM2;

#[doc(hidden)]
pub struct USART1;

/// The initial value of resources that were not given an initial value in `app.resources`
#[allow(non_snake_case)]
pub struct LateResources {
    /// Example of a resource that's initialized "late", or at runtime
    pub KEY: [u8; 128],
    _more: (),
}

/// Resources assigned to and owned by `init`
#[allow(non_snake_case)]
pub struct Resources {
    /// Example of a resource assigned to `init`
    pub BUFFER: BUFFER,
}

#[doc(hidden)]
pub struct BUFFER;

/// Tasks that `init` can schedule
pub struct Tasks {}
