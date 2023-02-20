pub use bare_metal::CriticalSection;
//pub use portable_atomic as atomic;
pub use atomic_polyfill as atomic;

pub mod executor;

#[cfg(all(
    cortex_m_basepri,
    not(any(feature = "thumbv7-backend", feature = "thumbv8main-backend"))
))]
compile_error!(
    "Building for Cortex-M with basepri, but 'thumbv7-backend' or 'thumbv8main-backend' backend not selected"
);

#[cfg(all(
    cortex_m_source_masking,
    not(any(feature = "thumbv6-backend", feature = "thumbv8base-backend"))
))]
compile_error!(
    "Building for Cortex-M with source masking, but 'thumbv6-backend' or 'thumbv8base-backend' backend not selected"
);

#[cfg(cortex_m_basepri)]
pub use cortex_basepri::*;

#[cfg(cortex_m_basepri)]
mod cortex_basepri;

#[cfg(cortex_m_source_masking)]
pub use cortex_source_mask::*;

#[cfg(cortex_m_source_masking)]
mod cortex_source_mask;

/// Priority conversion, takes logical priorities 1..=N and converts it to NVIC priority.
#[cfg(any(cortex_m_basepri, cortex_m_source_masking))]
#[inline]
#[must_use]
pub const fn cortex_logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}

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
