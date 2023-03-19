#[cfg(not(any(
    feature = "cortex-m-source-masking",
    feature = "cortex-m-basepri",
    feature = "riscv-e310x",
    feature = "test-template",
)))]
compile_error!("No backend selected");

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
pub use cortex::*;

#[cfg(feature = "riscv-e310x")]
pub use riscv_e310x::*;

#[cfg(feature = "test-template")]
pub use template::*;

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
mod cortex;

#[cfg(feature = "riscv-e310x")]
mod riscv_e310x;

#[cfg(feature = "test-template")]
mod template;
