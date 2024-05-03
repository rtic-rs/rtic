//! Macros that implement embedded-hal traits for Monotonics

/// Implements [`embedded_hal::delay::DelayNs`] for a given monotonic.
#[macro_export]
macro_rules! impl_embedded_hal_delay_fugit {
    ($t:ty) => {
        impl $crate::embedded_hal::delay::DelayNs for $t {
            fn delay_ns(&mut self, ns: u32) {
                let now = <Self as $crate::Monotonic>::now();
                let mut done =
                    now + <Self as $crate::Monotonic>::Duration::nanos_at_least(ns.into());
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += <Self as $crate::Monotonic>::Duration::from_ticks(1);
                }

                while <Self as $crate::Monotonic>::now() < done {}
            }

            fn delay_us(&mut self, us: u32) {
                let now = <Self as $crate::Monotonic>::now();
                let mut done =
                    now + <Self as $crate::Monotonic>::Duration::micros_at_least(us.into());
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done = done + <Self as $crate::Monotonic>::Duration::from_ticks(1);
                }

                while <Self as $crate::Monotonic>::now() < done {}
            }

            fn delay_ms(&mut self, ms: u32) {
                let now = <Self as $crate::Monotonic>::now();
                let mut done =
                    now + <Self as $crate::Monotonic>::Duration::millis_at_least(ms.into());
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done += <Self as $crate::Monotonic>::Duration::from_ticks(1);
                }

                while <Self as $crate::Monotonic>::now() < done {}
            }
        }
    };
}

/// Implements [`embedded_hal_async::delay::DelayNs`] for a given monotonic.
#[macro_export]
macro_rules! impl_embedded_hal_async_delay_fugit {
    ($t:ty) => {
        impl $crate::embedded_hal_async::delay::DelayNs for $t {
            #[inline]
            async fn delay_ns(&mut self, ns: u32) {
                <Self as $crate::Monotonic>::delay(
                    <Self as $crate::Monotonic>::Duration::nanos_at_least(ns.into()),
                )
                .await;
            }

            #[inline]
            async fn delay_us(&mut self, us: u32) {
                <Self as $crate::Monotonic>::delay(
                    <Self as $crate::Monotonic>::Duration::micros_at_least(us.into()),
                )
                .await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                <Self as $crate::Monotonic>::delay(
                    <Self as $crate::Monotonic>::Duration::millis_at_least(ms.into()),
                )
                .await;
            }
        }
    };
}
