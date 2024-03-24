//! A monotonic clock / counter definition.

/// # A monotonic clock / counter definition.
///
/// ## Correctness
///
/// The trait enforces that proper time-math is implemented between `Instant` and `Duration`. This
/// is a requirement on the time library that the user chooses to use.
pub trait Monotonic {
    /// The time at time zero.
    const ZERO: Self::Instant;

    /// The duration between two timer ticks.
    const TICK_PERIOD: Self::Duration;

    /// The type for instant, defining an instant in time.
    ///
    /// **Note:** In all APIs in RTIC that use instants from this monotonic, this type will be used.
    type Instant: Ord
        + Copy
        + core::ops::Add<Self::Duration, Output = Self::Instant>
        + core::ops::Sub<Self::Duration, Output = Self::Instant>
        + core::ops::Sub<Self::Instant, Output = Self::Duration>;

    /// The type for duration, defining an duration of time.
    ///
    /// **Note:** In all APIs in RTIC that use duration from this monotonic, this type will be used.
    type Duration;

    /// Get the current time.
    fn now() -> Self::Instant;

    /// Set the compare value of the timer interrupt.
    ///
    /// **Note:** This method does not need to handle race conditions of the monotonic, the timer
    /// queue in RTIC checks this.
    fn set_compare(instant: Self::Instant);

    /// This method used to be required by an errata workaround
    /// for the nrf52 family, but it has been disabled as the
    /// workaround was erroneous.
    #[deprecated(
        since = "1.2.0",
        note = "this method is erroneous and has been disabled"
    )]
    fn should_dequeue_check(_: Self::Instant) -> bool {
        panic!("This method should not be used as it is erroneous.")
    }

    /// Clear the compare interrupt flag.
    fn clear_compare_flag();

    /// Pend the timer's interrupt.
    fn pend_interrupt();

    /// Optional. Runs on interrupt before any timer queue handling.
    fn on_interrupt() {}

    /// Optional. This is used to save power, this is called when the timer queue is not empty.
    ///
    /// Enabling and disabling the monotonic needs to propagate to `now` so that an instant
    /// based of `now()` is still valid.
    ///
    /// NOTE: This may be called more than once.
    fn enable_timer() {}

    /// Optional. This is used to save power, this is called when the timer queue is empty.
    ///
    /// Enabling and disabling the monotonic needs to propagate to `now` so that an instant
    /// based of `now()` is still valid.
    ///
    /// NOTE: This may be called more than once.
    fn disable_timer() {}
}

/// Creates impl blocks for [`embedded_hal::delay::DelayNs`][DelayNs],
/// based on [`fugit::ExtU64Ceil`][ExtU64Ceil].
///
/// [DelayNs]: https://docs.rs/embedded-hal/latest/embedded_hal/delay/trait.DelayNs.html
/// [ExtU64Ceil]: https://docs.rs/fugit/latest/fugit/trait.ExtU64Ceil.html
#[macro_export]
macro_rules! embedded_hal_delay_impl_fugit64 {
    ($t:ty) => {
        impl ::embedded_hal::delay::DelayNs for $t {
            fn delay_ns(&mut self, ns: u32) {
                use ::fugit::ExtU64Ceil;

                let now = Self::now();
                let mut done = now + u64::from(ns).nanos_at_least();
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += Self::TICK_PERIOD;
                }

                while Self::now() < done {}
            }

            fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU64Ceil;

                let now = Self::now();
                let mut done = now + u64::from(us).micros_at_least();
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += Self::TICK_PERIOD;
                }

                while Self::now() < done {}
            }

            fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU64Ceil;

                let now = Self::now();
                let mut done = now + u64::from(ms).millis_at_least();
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += Self::TICK_PERIOD;
                }

                while Self::now() < done {}
            }
        }
    };
}

/// Creates impl blocks for [`embedded_hal_async::delay::DelayNs`][DelayNs],
/// based on [`fugit::ExtU64Ceil`][ExtU64Ceil].
///
/// [DelayNs]: https://docs.rs/embedded-hal-async/latest/embedded_hal_async/delay/trait.DelayNs.html
/// [ExtU64Ceil]: https://docs.rs/fugit/latest/fugit/trait.ExtU64Ceil.html
#[macro_export]
macro_rules! embedded_hal_async_delay_impl_fugit64 {
    ($t:ty) => {
        impl ::embedded_hal_async::delay::DelayNs for $t {
            #[inline]
            async fn delay_ns(&mut self, ns: u32) {
                use ::fugit::ExtU64Ceil;
                Self::delay(u64::from(ns).nanos_at_least()).await;
            }

            #[inline]
            async fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU64Ceil;
                Self::delay(u64::from(us).micros_at_least()).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU64Ceil;
                Self::delay(u64::from(ms).millis_at_least()).await;
            }
        }
    };
}

/// Creates impl blocks for [`embedded_hal::delay::DelayNs`][DelayNs],
/// based on [`fugit::ExtU32Ceil`][ExtU32Ceil].
///
/// [DelayNs]: https://docs.rs/embedded-hal/latest/embedded_hal/delay/trait.DelayNs.html
/// [ExtU32Ceil]: https://docs.rs/fugit/latest/fugit/trait.ExtU32Ceil.html
#[macro_export]
macro_rules! embedded_hal_delay_impl_fugit32 {
    ($t:ty) => {
        impl ::embedded_hal::delay::DelayNs for $t {
            fn delay_ns(&mut self, ns: u32) {
                use ::fugit::ExtU32Ceil;

                let now = Self::now();
                let mut done = now + ns.nanos_at_least();
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += Self::TICK_PERIOD;
                }

                while Self::now() < done {}
            }

            fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU32Ceil;

                let now = Self::now();
                let mut done = now + us.micros_at_least();
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += Self::TICK_PERIOD;
                }

                while Self::now() < done {}
            }

            fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU32Ceil;

                let now = Self::now();
                let mut done = now + ms.millis_at_least();
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += Self::TICK_PERIOD;
                }

                while Self::now() < done {}
            }
        }
    };
}

/// Creates impl blocks for [`embedded_hal_async::delay::DelayNs`][DelayNs],
/// based on [`fugit::ExtU32Ceil`][ExtU32Ceil].
///
/// [DelayNs]: https://docs.rs/embedded-hal-async/latest/embedded_hal_async/delay/trait.DelayNs.html
/// [ExtU32Ceil]: https://docs.rs/fugit/latest/fugit/trait.ExtU32Ceil.html
#[macro_export]
macro_rules! embedded_hal_async_delay_impl_fugit32 {
    ($t:ty) => {
        impl ::embedded_hal_async::delay::DelayNs for $t {
            #[inline]
            async fn delay_ns(&mut self, ns: u32) {
                use ::fugit::ExtU32Ceil;
                Self::delay(ns.nanos_at_least()).await;
            }

            #[inline]
            async fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU32Ceil;
                Self::delay(us.micros_at_least()).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU32Ceil;
                Self::delay(ms.millis_at_least()).await;
            }
        }
    };
}
