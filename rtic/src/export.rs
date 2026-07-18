pub use critical_section::CriticalSection;
pub use portable_atomic as atomic;

pub mod executor;

// Cortex-M target (any)
#[cfg(any(
    implementation = "cortex-m-basepri",
    implementation = "cortex-m-source-masking"
))]
pub use cortex_common::*;

#[cfg(any(
    implementation = "cortex-m-basepri",
    implementation = "cortex-m-source-masking"
))]
mod cortex_common;

// Cortex-M target with basepri support
#[cfg(implementation = "cortex-m-basepri")]
mod cortex_basepri;

#[cfg(implementation = "cortex-m-basepri")]
pub use cortex_basepri::*;

// Cortex-M target with source mask support
#[cfg(implementation = "cortex-m-source-masking")]
mod cortex_source_mask;

#[cfg(implementation = "cortex-m-source-masking")]
pub use cortex_source_mask::*;

#[cfg(any(
    implementation = "riscv-esp32c3",
    implementation = "riscv-esp32c6",
    implementation = "riscv-slic"
))]
pub mod riscv_common;

#[cfg(any(
    implementation = "riscv-esp32c3",
    implementation = "riscv-esp32c6",
    implementation = "riscv-slic"
))]
pub use riscv_common::*;

#[cfg(implementation = "riscv-esp32c3")]
mod riscv_esp32c3;
#[cfg(implementation = "riscv-esp32c3")]
pub use riscv_esp32c3::*;

#[cfg(implementation = "riscv-esp32c6")]
mod riscv_esp32c6;
#[cfg(implementation = "riscv-esp32c6")]
pub use riscv_esp32c6::*;

#[cfg(implementation = "riscv-slic")]
mod slic;
#[cfg(implementation = "riscv-slic")]
pub use slic::*;

#[inline(always)]
pub fn assert_send<T: Send>() {}

#[inline(always)]
pub fn assert_sync<T: Sync>() {}
