pub use bare_metal::CriticalSection;
//pub use portable_atomic as atomic;
pub use atomic_polyfill as atomic;

pub mod executor;

#[cfg(feature = "cortex-m")]
pub use cortex_backend::*;

#[cfg(feature = "cortex-m")]
mod cortex_backend;

#[cfg(feature = "riscv")]
pub use riscv_backend::*;

#[cfg(feature = "riscv")]
mod riscv_backend;

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
