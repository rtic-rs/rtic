//! [`Monotonic`](super::Monotonic) implementations for the nRF series of MCUs.

// An nRF91 chip can be driven through either its secure (`-s`) or its non-secure
// (`-ns`) peripheral alias, but not both: each variant re-exports the same RTC and
// TIMER instances, so enabling both would be ambiguous. Catch that misconfiguration
// with a clear message instead of a cryptic duplicate-import error.
#[cfg(all(
    any(feature = "nrf9160-s", feature = "nrf9151-s", feature = "nrf9161-s"),
    any(feature = "nrf9160-ns", feature = "nrf9151-ns", feature = "nrf9161-ns")
))]
compile_error!(
    "rtic-monotonics: enable only one nRF91 security variant - either a secure (`-s`) \
     or a non-secure (`-ns`) feature, not both."
);

pub mod rtc;
pub mod timer;
