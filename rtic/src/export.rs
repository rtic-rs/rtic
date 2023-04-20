pub use bare_metal::CriticalSection;
//pub use portable_atomic as atomic;
pub use atomic_polyfill as atomic;

pub mod executor;


// Cortex-M target (any)
#[cfg(feature = "cortex-m")]
pub use cortex_common::*;

#[cfg(feature = "cortex-m")]
mod cortex_common;

// Cortex-M target with basepri support
#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
mod cortex_basepri;

#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
pub use cortex_basepri::*;

// Cortex-M target with source mask support
#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv6"))]
mod cortex_source_mask;

#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv6"))]
pub use cortex_source_mask::*;

// RISC-V target (any)
#[cfg(feature = "riscv")]
pub use riscv_common::*;

#[cfg(feature = "riscv")]
mod riscv_common;

#[cfg(feature = "riscv-esp32c3")]
mod riscv_esp32c3;
#[cfg(feature = "riscv-esp32c3")]
pub use riscv_esp32c3::*;


#[inline(always)]
pub fn assert_send<T>()
where
    T: Send,
{
}

#[inline(always)]
pub fn assert_sync<T>()
where
    T: Sync,
{
}
