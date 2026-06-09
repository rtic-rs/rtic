//! [`Monotonic`](super::Monotonic) implementations for the SiLabs EFR32/EFM32
//! series.
//!
//! * [rtcc]: RTCC peripheral (32-bit).
//! * [letimer]: LETimer (24-bit), low-frequency clock that runs in EM2+ deep sleep.
//! * [timer]: TIMER0 (32-bit), high-frequency clock prescaled to a chosen tick rate.

pub mod letimer;
#[cfg(feature = "silabs-efr32mg22")]
pub mod rtcc;
#[cfg(any(
    feature = "silabs-efr32mg24",
    feature = "silabs-efr32fg25",
    feature = "silabs-efr32mg26"
))]
pub mod timer;

const NVIC_PRIO_BITS: u8 = 4;
