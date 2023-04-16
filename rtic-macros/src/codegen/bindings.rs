#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
pub use cortex::*;

#[cfg(feature = "test-template")]
pub use template::*;

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
mod cortex;

#[cfg(feature = "test-template")]
mod template;
