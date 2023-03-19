pub use bare_metal::CriticalSection;
//pub use portable_atomic as atomic;
pub use atomic_polyfill as atomic;

pub mod executor;

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

#[cfg(all(feature = "riscv-e310x", not(any(feature = "riscv-e310x-backend"))))]
compile_error!("Building for RISC-V E310X, but 'riscv-e310x-backend' backend not selected");

#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
pub use cortex_basepri::*;

#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
mod cortex_basepri;

#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv6"))]
pub use cortex_source_mask::*;

#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv6"))]
mod cortex_source_mask;

/// Priority conversion, takes logical priorities 1..=N and converts it to NVIC priority.
#[cfg(any(
    feature = "cortex-m-basepri",
    feature = "cortex-m-source-masking",
    feature = "rtic-uitestv6",
    feature = "rtic-uitestv7",
))]
#[inline]
#[must_use]
pub const fn cortex_logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}

#[cfg(any(feature = "riscv-e310x"))]
pub use riscv_e310x::*;

#[cfg(any(feature = "riscv-e310x"))]
mod riscv_e310x;

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
