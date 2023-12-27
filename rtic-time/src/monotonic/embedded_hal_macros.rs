//! Macros that implement embedded-hal traits for Monotonics

/// Implements [`embedded_hal::delay::DelayNs`] for a given monotonic.
#[macro_export]
macro_rules! implement_embedded_hal_delay_trait {
    (
        $t:ty,
        millis_at_least: $millis_at_least:expr,
        micros_at_least: $micros_at_least:expr,
        nanos_at_least: $nanos_at_least:expr $(,)?
    ) => {
        impl ::embedded_hal::delay::DelayNs for $t {
            fn delay_ns(&mut self, ns: u32) {
                let now = <Self as $crate::monotonic::Monotonic>::now();
                let mut done = now + ($nanos_at_least)(ns.into());
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done = done + <Self as $crate::monotonic::Monotonic>::DURATION_ONE;
                }

                while <Self as $crate::monotonic::Monotonic>::now() < done {}
            }

            fn delay_us(&mut self, us: u32) {
                let now = <Self as $crate::monotonic::Monotonic>::now();
                let mut done = now + ($micros_at_least)(us.into());
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done = done + <Self as $crate::monotonic::Monotonic>::DURATION_ONE;
                }

                while <Self as $crate::monotonic::Monotonic>::now() < done {}
            }

            fn delay_ms(&mut self, ms: u32) {
                let now = <Self as $crate::monotonic::Monotonic>::now();
                let mut done = now + ($millis_at_least)(ms.into());
                if now != done {
                    // Compensate for sub-tick uncertainty
                    done = done + <Self as $crate::monotonic::Monotonic>::DURATION_ONE;
                }

                while <Self as $crate::monotonic::Monotonic>::now() < done {}
            }
        }
    };
}

/// Implements [`embedded_hal_async::delay::DelayNs`] for a given monotonic.
#[macro_export]
macro_rules! implement_embedded_hal_async_delay_trait {
    (
        $t:ty,
        millis_at_least: $millis_at_least:expr,
        micros_at_least: $micros_at_least:expr,
        nanos_at_least: $nanos_at_least:expr $(,)?
    ) => {
        impl ::embedded_hal_async::delay::DelayNs for $t {
            #[inline]
            async fn delay_ns(&mut self, ns: u32) {
                <Self as $crate::monotonic::Monotonic>::delay(($nanos_at_least)(ns.into())).await;
            }

            #[inline]
            async fn delay_us(&mut self, us: u32) {
                <Self as $crate::monotonic::Monotonic>::delay(($micros_at_least)(us.into())).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                <Self as $crate::monotonic::Monotonic>::delay(($millis_at_least)(ms.into())).await;
            }
        }
    };
}
