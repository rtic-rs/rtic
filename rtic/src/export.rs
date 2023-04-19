pub use bare_metal::CriticalSection;
//pub use portable_atomic as atomic;
pub use atomic_polyfill as atomic;

pub mod executor;

// cortex-m common
#[cfg(feature = "cortex-m")]
pub use cortex_backend::*;

#[cfg(feature = "cortex-m")]
mod cortex_backend;

// basepri support
#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
mod cortex_basepri;

#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
pub use cortex_basepri::*;
// source mask support
#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv7"))]
mod cortex_source_mask;

#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv7"))]
pub use cortex_source_mask::*;

#[cfg(all(
    feature = "cortex-m-basepri",
    not(any(feature = "thumbv7-backend", feature = "thumbv8main-backend"))
))]
compile_error!(
    "Building for Cortex-M with basepri, but 'thumbv7-backend' or 'thumbv8main-backend' backend not selected"
);

#[cfg(all(
    feature = "cortex-m-source-masking",
    not(any(feature = "thumbv6-backend", feature = "thumbv8base-backend"))
))]
compile_error!(
    "Building for Cortex-M with source masking, but 'thumbv6-backend' or 'thumbv8base-backend' backend not selected"
);

// risc-v common
#[cfg(feature = "riscv")]
pub use riscv_common::*;

#[cfg(feature = "riscv")]
mod riscv_common;

// slic support
#[cfg(feature = "riscv-slic")]
pub use self::riscv_slic::*;

#[cfg(feature = "riscv-slic")]
pub mod riscv_slic;

#[cfg(all(feature = "riscv-slic", not(any(feature = "riscv-slic-backend"))))]
compile_error!("Building for RISC-V SLIC, but 'riscv-slic-backend' backend not selected");

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
