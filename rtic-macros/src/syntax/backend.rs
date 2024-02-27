#[cfg(not(any(
    feature = "cortex-m-source-masking",
    feature = "cortex-m-basepri",
    feature = "test-template",
    feature = "riscv-esp32c3",
    feature = "riscv-slic",
)))]
compile_error!("No backend selected");

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
pub use cortex::*;

#[cfg(feature = "test-template")]
pub use template::*;

#[cfg(feature = "riscv-esp32c3")]
pub use esp32c3::*;

#[cfg(feature = "riscv-slic")]
pub use riscv_slic::*;

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
mod cortex;

#[cfg(feature = "test-template")]
mod template;

#[cfg(feature = "riscv-esp32c3")]
mod esp32c3;

#[cfg(feature = "riscv-slic")]
mod riscv_slic;
