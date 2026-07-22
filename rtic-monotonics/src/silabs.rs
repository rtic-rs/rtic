//! [`Monotonic`](super::Monotonic) implementations for the SiLabs EFR32/EFM32
//! series.
//!
//! * [rtcc]: RTCC peripheral (32-bit).
//! * [letimer]: LETimer (24-bit), low-frequency clock that runs in EM2+ deep sleep.
//! * [timer]: TIMER0 (32-bit), high-frequency clock prescaled to a chosen tick rate.

#[cfg(feature = "silabs_letimer0")]
pub mod letimer;
#[cfg(feature = "silabs_rtcc")]
pub mod rtcc;
#[cfg(any(
    feature = "silabs_timer0",
    feature = "silabs_timer1",
    feature = "silabs_timer2",
    feature = "silabs_timer3",
    feature = "silabs_timer4",
    feature = "silabs_timer5",
    feature = "silabs_timer6",
    feature = "silabs_timer7",
    feature = "silabs_timer8",
    feature = "silabs_timer9",
))]
pub mod timer;

const NVIC_PRIO_BITS: u8 = 4;
