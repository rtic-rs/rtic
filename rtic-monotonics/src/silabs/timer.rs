//! [`Monotonic`](rtic_time::Monotonic) implementations for the EFR32 `TIMER`
//! peripherals.
//!
//! Each `TIMERn` instance has its own backend, selected by the matching `silabs_timerN` feature.
//! Runs on the high-frequency `EM01GRPACLK`, prescaled to a chosen tick rate.
//!
//! # Example
//!
//! ```ignore
//! use rtic_monotonics::silabs::timer::prelude::*;
//!
//! // 1 MHz tick rate.
//! silabs_timer0_monotonic!(Mono, 1_000_000);
//!
//! fn init() {
//!     // `tim_clock_hz` is the EM01GRPACLK frequency feeding the timer.
//!     Mono::start(40_000_000);
//! }
//!
//! async fn usage() {
//!     loop {
//!         let timestamp = Mono::now();
//!         Mono::delay(100.millis()).await;
//!     }
//! }
//! ```

/// Common imports for using the silabs TIMER monotonics.
pub mod prelude {
    #[cfg(feature = "silabs_timer0")]
    pub use crate::silabs_timer0_monotonic;
    #[cfg(feature = "silabs_timer1")]
    pub use crate::silabs_timer1_monotonic;
    #[cfg(feature = "silabs_timer2")]
    pub use crate::silabs_timer2_monotonic;
    #[cfg(feature = "silabs_timer3")]
    pub use crate::silabs_timer3_monotonic;
    #[cfg(feature = "silabs_timer4")]
    pub use crate::silabs_timer4_monotonic;
    #[cfg(feature = "silabs_timer5")]
    pub use crate::silabs_timer5_monotonic;
    #[cfg(feature = "silabs_timer6")]
    pub use crate::silabs_timer6_monotonic;
    #[cfg(feature = "silabs_timer7")]
    pub use crate::silabs_timer7_monotonic;
    #[cfg(feature = "silabs_timer8")]
    pub use crate::silabs_timer8_monotonic;
    #[cfg(feature = "silabs_timer9")]
    pub use crate::silabs_timer9_monotonic;

    pub use crate::Monotonic;

    pub use crate::fugit::{self, ExtU64, ExtU64Ceil};
}

// Shared imports, present only when at least one instance is selected.
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
use {
    crate::set_monotonic_prio,
    crate::silabs::NVIC_PRIO_BITS,
    cortex_m::peripheral::NVIC,
    portable_atomic::AtomicU64,
    portable_atomic::Ordering,
    rtic_time::half_period_counter::calculate_now,
    rtic_time::timer_queue::TimerQueue,
    silabs_metapac::Interrupt,
    word::{compute_compare_value, half_period_value, timer_max},
};

pub use crate::TimerQueueBackend;

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_silabs_timer_monotonic {
    ($name:ident, $backend:ident, $irq:ident, $tick_rate_hz:expr) => {
        /// A `Monotonic` based on an EFR32 TIMER peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// `tim_clock_hz` is the EM01GRPACLK frequency feeding the timer.
            /// This method must be called only once.
            pub fn start(tim_clock_hz: u32) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn $irq() {
                    use $crate::TimerQueueBackend;
                    $crate::silabs::timer::$backend::timer_queue().on_monotonic_interrupt();
                }

                $crate::silabs::timer::$backend::_start(tim_clock_hz, $tick_rate_hz);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::silabs::timer::$backend;
            type Instant = $crate::fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
            type Duration = $crate::fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}

/// Create a `TIMER0`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer0")]
#[macro_export]
macro_rules! silabs_timer0_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer0Backend, TIMER0, $tick_rate_hz);
    };
}
/// Create a `TIMER1`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer1")]
#[macro_export]
macro_rules! silabs_timer1_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer1Backend, TIMER1, $tick_rate_hz);
    };
}
/// Create a `TIMER2`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer2")]
#[macro_export]
macro_rules! silabs_timer2_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer2Backend, TIMER2, $tick_rate_hz);
    };
}
/// Create a `TIMER3`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer3")]
#[macro_export]
macro_rules! silabs_timer3_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer3Backend, TIMER3, $tick_rate_hz);
    };
}
/// Create a `TIMER4`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer4")]
#[macro_export]
macro_rules! silabs_timer4_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer4Backend, TIMER4, $tick_rate_hz);
    };
}
/// Create a `TIMER5`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer5")]
#[macro_export]
macro_rules! silabs_timer5_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer5Backend, TIMER5, $tick_rate_hz);
    };
}
/// Create a `TIMER6`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer6")]
#[macro_export]
macro_rules! silabs_timer6_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer6Backend, TIMER6, $tick_rate_hz);
    };
}
/// Create a `TIMER7`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer7")]
#[macro_export]
macro_rules! silabs_timer7_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer7Backend, TIMER7, $tick_rate_hz);
    };
}
/// Create a `TIMER8`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer8")]
#[macro_export]
macro_rules! silabs_timer8_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer8Backend, TIMER8, $tick_rate_hz);
    };
}
/// Create a `TIMER9`-based monotonic and register its interrupt. See [`crate::silabs::timer`].
#[cfg(feature = "silabs_timer9")]
#[macro_export]
macro_rules! silabs_timer9_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_silabs_timer_monotonic!($name, Timer9Backend, TIMER9, $tick_rate_hz);
    };
}

/// The closure handed to [`half_period_value`]/[`timer_max`]/
/// [`compute_compare_value`] is never called — it forces those values to the
/// same width as the `CNT` register read, so a wrong-width register access (or
/// reading the raw register instead of the counter field) fails to compile
/// instead of silently miscomputing the half-period.
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
mod word {
    pub trait TimerWord: TryFrom<u64> {
        const HALF_PERIOD_VALUE: Self;
        fn trunc_from_u64(val: u64) -> Self;
    }

    impl TimerWord for u16 {
        const HALF_PERIOD_VALUE: Self = 0x8000;
        fn trunc_from_u64(val: u64) -> Self {
            val as Self
        }
    }

    impl TimerWord for u32 {
        const HALF_PERIOD_VALUE: Self = 0x8000_0000;
        fn trunc_from_u64(val: u64) -> Self {
            val as Self
        }
    }

    /// Returns the half-period value for a timer.
    ///
    /// The closure argument is never invoked; it forces the CNT and CC0_OC word
    /// types to be inferred as the same `T`, causing a width mismatch between
    /// the two registers to fail type-checking.
    ///
    /// # Example
    ///
    /// ```ignore
    /// timer.cc0_oc().write(|w| w.set_oc(half_period_value(|| timer.cnt().read().cnt())));
    /// ```
    pub fn half_period_value<T: TimerWord>(_cnt: impl FnOnce() -> T) -> T {
        T::HALF_PERIOD_VALUE
    }

    /// TOP (auto-reload) value — all-ones at the counter width, inferred from
    /// the `CNT` read closure.
    pub fn timer_max<T: TimerWord>(_cnt: impl FnOnce() -> T) -> T {
        T::trunc_from_u64(u64::MAX)
    }

    /// Computes the CC1_OC compare value for a target `instant`, given the
    /// current counter value `now`.
    ///
    /// If the target is in the past or would overflow the timer's range before
    /// being reached, returns `0` so the compare match fires on the next
    /// wrap-around rather than at a stale value.
    ///
    /// The closure argument is never invoked; it forces the CNT and CC1_OC word
    /// types to be inferred as the same `T`, causing a width mismatch between
    /// the two registers to fail type-checking.
    ///
    /// # Example
    ///
    /// ```ignore
    /// timer.cc1_oc().write(|w| {
    ///     w.set_oc(compute_compare_value(instant, now, || timer.cnt().read().cnt()));
    /// });
    /// ```
    pub fn compute_compare_value<T: TimerWord>(
        instant: u64,
        now: u64,
        _cnt: impl FnOnce() -> T,
    ) -> T {
        // Since the timer may or may not overflow based on the requested compare val, we check how many ticks are left.
        // `wrapping_sub` takes care of the u64 integer overflow special case.
        let val = if T::try_from(instant.wrapping_sub(now)).is_ok() {
            instant
        } else {
            // In the past or will overflow
            0
        };
        T::trunc_from_u64(val)
    }
}

/// Generate a `TimerQueueBackend` for one `TIMERn` instance.
///
/// `$vals` selects the register block (16- vs 32-bit) and `$clken`/`$set_timer`
/// the bus-clock gate; the counter width is inferred from the `CNT` read.
macro_rules! make_silabs_timer {
    ($backend:ident, $timer:ident, $vals:path,
     $clken:ident, $set_timer:ident, $overflow:ident, $tq:ident) => {
        /// TIMER-based [`TimerQueueBackend`].
        pub struct $backend;

        static $overflow: AtomicU64 = AtomicU64::new(0);
        static $tq: TimerQueue<$backend> = TimerQueue::new();

        impl $backend {
            /// Starts the timer.
            ///
            /// **Do not use this function directly.** Use the prelude macro.
            pub fn _start(tim_clock_hz: u32, tick_rate_hz: u32) {
                use $vals::{Cc0CfgMode, Cc1CfgMode, Presc};
                let t = silabs_metapac::$timer;

                silabs_metapac::CMU.$clken().modify(|w| w.$set_timer(true));

                // CFG / CCx_CFG are writable only while EN = 0; the rest needs EN = 1.
                t.en().write(|w| w.set_en(false));
                while t.en().read().disabling() {}

                assert!(
                    tim_clock_hz % tick_rate_hz == 0,
                    "TIMER clock is not an integer multiple of the desired tick rate"
                );
                let psc = u16::try_from(tim_clock_hz / tick_rate_hz - 1)
                    .expect("TIMER prescaler out of range (max divisor 1024)");
                t.cfg().write(|w| w.set_presc(Presc::from_bits(psc)));

                t.cc0_cfg().write(|w| w.set_mode(Cc0CfgMode::Outputcompare));
                t.cc1_cfg().write(|w| w.set_mode(Cc1CfgMode::Outputcompare));

                t.en().write(|w| w.set_en(true));
                t.cmd().write(|w| w.set_stop(true));
                // TOP and the half-period marker are inferred from the CNT read;
                // a wrong-width register access fails to compile.
                t.top()
                    .write(|w| w.set_top(timer_max(|| silabs_metapac::$timer.cnt().read().cnt())));
                t.cnt().write(|w| w.set_cnt(1));
                t.cc0_oc().write(|w| {
                    w.set_oc(half_period_value(|| {
                        silabs_metapac::$timer.cnt().read().cnt()
                    }))
                });

                t.if_clr().write(|w| {
                    w.set_of(true);
                    w.set_cc0(true);
                    w.set_cc1(true);
                });
                t.ien().write(|w| {
                    w.set_of(true);
                    w.set_cc0(true);
                });

                $tq.initialize(Self {});
                $overflow.store(0, Ordering::SeqCst);
                t.cmd().write(|w| w.set_start(true));

                unsafe {
                    set_monotonic_prio(NVIC_PRIO_BITS, Interrupt::$timer);
                    NVIC::unmask(Interrupt::$timer);
                }
            }
        }

        impl TimerQueueBackend for $backend {
            type Ticks = u64;

            fn now() -> Self::Ticks {
                calculate_now(
                    || $overflow.load(Ordering::Relaxed),
                    || silabs_metapac::$timer.cnt().read().cnt(),
                )
            }

            fn set_compare(instant: Self::Ticks) {
                let now = Self::now();
                silabs_metapac::$timer.cc1_oc().write(|w| {
                    w.set_oc(compute_compare_value(instant, now, || {
                        silabs_metapac::$timer.cnt().read().cnt()
                    }))
                });
            }

            fn clear_compare_flag() {
                silabs_metapac::$timer.if_clr().write(|w| w.set_cc1(true));
            }

            fn pend_interrupt() {
                NVIC::pend(Interrupt::$timer);
            }

            fn enable_timer() {
                silabs_metapac::$timer.ien_set().write(|w| w.set_cc1(true));
            }

            fn disable_timer() {
                silabs_metapac::$timer.ien_clr().write(|w| w.set_cc1(true));
            }

            fn on_interrupt() {
                let t = silabs_metapac::$timer;
                let flags = t.if_().read();
                // Full period (overflow at TOP).
                if flags.of() {
                    t.if_clr().write(|w| w.set_of(true));
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 1, "Monotonic must have skipped an interrupt!");
                }
                // Half period (CC0).
                if flags.cc0() {
                    t.if_clr().write(|w| w.set_cc0(true));
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 0, "Monotonic must have skipped an interrupt!");
                }
            }

            fn timer_queue() -> &'static TimerQueue<Self> {
                &$tq
            }
        }
    };
}

// 32-bit (timer_v1_w), CLKEN0
#[cfg(feature = "silabs_timer0")]
make_silabs_timer!(
    Timer0Backend,
    TIMER0,
    silabs_metapac::timer_v1_w::vals,
    clken0,
    set_timer0,
    TIMER0_OVERFLOW,
    TIMER0_TQ
);
#[cfg(feature = "silabs_timer1")]
make_silabs_timer!(
    Timer1Backend,
    TIMER1,
    silabs_metapac::timer_v1_w::vals,
    clken0,
    set_timer1,
    TIMER1_OVERFLOW,
    TIMER1_TQ
);
// 16-bit (timer_v1), CLKEN0
#[cfg(feature = "silabs_timer2")]
make_silabs_timer!(
    Timer2Backend,
    TIMER2,
    silabs_metapac::timer_v1::vals,
    clken0,
    set_timer2,
    TIMER2_OVERFLOW,
    TIMER2_TQ
);
#[cfg(feature = "silabs_timer3")]
make_silabs_timer!(
    Timer3Backend,
    TIMER3,
    silabs_metapac::timer_v1::vals,
    clken0,
    set_timer3,
    TIMER3_OVERFLOW,
    TIMER3_TQ
);
#[cfg(feature = "silabs_timer4")]
make_silabs_timer!(
    Timer4Backend,
    TIMER4,
    silabs_metapac::timer_v1::vals,
    clken0,
    set_timer4,
    TIMER4_OVERFLOW,
    TIMER4_TQ
);
// 16-bit (timer_v1), CLKEN2
#[cfg(feature = "silabs_timer5")]
make_silabs_timer!(
    Timer5Backend,
    TIMER5,
    silabs_metapac::timer_v1::vals,
    clken2,
    set_timer5,
    TIMER5_OVERFLOW,
    TIMER5_TQ
);
#[cfg(feature = "silabs_timer6")]
make_silabs_timer!(
    Timer6Backend,
    TIMER6,
    silabs_metapac::timer_v1::vals,
    clken2,
    set_timer6,
    TIMER6_OVERFLOW,
    TIMER6_TQ
);
#[cfg(feature = "silabs_timer7")]
make_silabs_timer!(
    Timer7Backend,
    TIMER7,
    silabs_metapac::timer_v1::vals,
    clken2,
    set_timer7,
    TIMER7_OVERFLOW,
    TIMER7_TQ
);
// 32-bit (timer_v1_w), CLKEN2
#[cfg(feature = "silabs_timer8")]
make_silabs_timer!(
    Timer8Backend,
    TIMER8,
    silabs_metapac::timer_v1_w::vals,
    clken2,
    set_timer8,
    TIMER8_OVERFLOW,
    TIMER8_TQ
);
#[cfg(feature = "silabs_timer9")]
make_silabs_timer!(
    Timer9Backend,
    TIMER9,
    silabs_metapac::timer_v1_w::vals,
    clken2,
    set_timer9,
    TIMER9_OVERFLOW,
    TIMER9_TQ
);
