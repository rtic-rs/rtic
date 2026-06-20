//! [`Monotonic`](super::Monotonic) implementations for the SiLabs series of
//! MCUs (EFR32, EFM32, ...).
//!
//! There are two monotonic implementations:
//! * [rtcc]: uses the RTCC peripheral (32-bit), only supported on the EFR32MG22
//! * [letimer]: uses the LETimer peripheral (24-bit)

pub mod letimer;
#[cfg(feature = "silabs-efr32mg22")]
pub mod rtcc;

const NVIC_PRIO_BITS: u8 = 4;
