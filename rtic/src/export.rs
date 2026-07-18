pub use critical_section::CriticalSection;
pub use portable_atomic as atomic;

pub mod executor;

// Cortex-M target (any)
#[cfg(feature = "cortex-m")]
pub use cortex_common::*;

#[cfg(feature = "cortex-m")]
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

#[cfg(feature = "riscv")]
pub mod riscv_common;

#[cfg(feature = "riscv")]
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
