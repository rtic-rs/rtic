//! [`Monotonic`](rtic_time::Monotonic) implementation for the EFR32 32-bit `TIMER0` peripheral.
//!
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
//!     // `tim_clock_hz` is the EM01GRPACLK frequency feeding TIMER0.
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

/// Common imports for using the silabs TIMER monotonic.
pub mod prelude {
    pub use crate::silabs_timer0_monotonic;

    pub use crate::Monotonic;

    pub use crate::fugit::{self, ExtU64, ExtU64Ceil};
}

use crate::set_monotonic_prio;
use crate::silabs::NVIC_PRIO_BITS;
pub use crate::TimerQueueBackend;
use cortex_m::peripheral::NVIC;
use portable_atomic::{AtomicU64, Ordering};
use rtic_time::half_period_counter::calculate_now;
use rtic_time::timer_queue::TimerQueue;
use silabs_metapac::timer_v1_w::vals::{Cc0CfgMode, Cc1CfgMode, Presc};
use silabs_metapac::{Interrupt, CMU, TIMER0};

/// Half of the 32-bit counter range — the half-period interrupt marker.
const HALF_PERIOD: u32 = 0x8000_0000;

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

static TIMER0_HALF_PERIODS: AtomicU64 = AtomicU64::new(0);
static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();

/// CC1 compare value for `instant`.
/// 0 (next wrap) if it is in the past or more than one period away.
fn compute_compare_value(instant: u64, now: u64) -> u32 {
    if u32::try_from(instant.wrapping_sub(now)).is_ok() {
        instant as u32
    } else {
        0
    }
}

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.** Use the prelude macro instead.
    ///
    /// * `tim_clock_hz` - frequency of the EM01GRPACLK branch feeding TIMER0.
    /// * `tick_rate_hz` - desired monotonic tick rate. `tim_clock_hz` must be
    ///   an integer multiple of it, and the resulting prescaler divisor must
    ///   fit in `1..=1024`.
    pub fn _start(tim_clock_hz: u32, tick_rate_hz: u32) {
        // Enable the TIMER0 bus clock.
        CMU.clken0().modify(|w| w.set_timer0(true));

        // Series 2 access rules: CFG / CCx_CFG are writable only while EN = 0;
        // every other register requires EN = 1.
        TIMER0.en().write(|w| w.set_en(false));
        while TIMER0.en().read().disabling() {}

        assert!(
            tim_clock_hz % tick_rate_hz == 0,
            "TIMER0 clock is not an integer multiple of the desired tick rate"
        );
        let psc = u16::try_from(tim_clock_hz / tick_rate_hz - 1)
            .expect("TIMER0 prescaler out of range (max divisor 1024)");
        TIMER0.cfg().write(|w| w.set_presc(Presc::from_bits(psc)));

        // CC0 is the half-period marker, CC1 is the alarm/compare slot.
        TIMER0
            .cc0_cfg()
            .write(|w| w.set_mode(Cc0CfgMode::Outputcompare));
        TIMER0
            .cc1_cfg()
            .write(|w| w.set_mode(Cc1CfgMode::Outputcompare));

        TIMER0.en().write(|w| w.set_en(true));

        TIMER0.cmd().write(|w| w.set_stop(true));
        TIMER0.top().write(|w| w.set_top(u32::MAX));
        // Start in the first half-period (period counter even, counter < HALF).
        TIMER0.cnt().write(|w| w.set_cnt(1));
        TIMER0.cc0_oc().write(|w| w.set_oc(HALF_PERIOD));

        // Clear any flags raised while configuring.
        TIMER0.if_clr().write(|w| {
            w.set_of(true);
            w.set_cc0(true);
            w.set_cc1(true);
        });

        // Enable the overflow (full-period) and CC0 (half-period) interrupts.
        TIMER0.ien().write(|w| {
            w.set_of(true);
            w.set_cc0(true);
        });

        TIMER_QUEUE.initialize(Self {});
        TIMER0_HALF_PERIODS.store(0, Ordering::SeqCst);

        TIMER0.cmd().write(|w| w.set_start(true));

        // SAFETY: we own TIMER0 and its interrupt vector and use no external
        // shared resources, so we don't disturb basepri-based critical sections.
        unsafe {
            set_monotonic_prio(NVIC_PRIO_BITS, Interrupt::TIMER0);
            NVIC::unmask(Interrupt::TIMER0);
        }
    }
}

impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        calculate_now(
            || TIMER0_HALF_PERIODS.load(Ordering::Relaxed),
            || TIMER0.cnt().read().cnt(),
        )
    }

    fn set_compare(instant: Self::Ticks) {
        let now = Self::now();
        TIMER0
            .cc1_oc()
            .write(|w| w.set_oc(compute_compare_value(instant, now)));
    }

    fn clear_compare_flag() {
        TIMER0.if_clr().write(|w| w.set_cc1(true));
    }

    fn pend_interrupt() {
        NVIC::pend(Interrupt::TIMER0);
    }

    fn enable_timer() {
        TIMER0.ien_set().write(|w| w.set_cc1(true));
    }

    fn disable_timer() {
        TIMER0.ien_clr().write(|w| w.set_cc1(true));
    }

    fn on_interrupt() {
        let flags = TIMER0.if_().read();
        // Full period - overflow at TOP.
        if flags.of() {
            TIMER0.if_clr().write(|w| w.set_of(true));
            let prev = TIMER0_HALF_PERIODS.fetch_add(1, Ordering::Relaxed);
            assert!(prev % 2 == 1, "Monotonic must have missed an interrupt!");
        }
        // Half period - CC0 compare at HALF_PERIOD.
        if flags.cc0() {
            TIMER0.if_clr().write(|w| w.set_cc0(true));
            let prev = TIMER0_HALF_PERIODS.fetch_add(1, Ordering::Relaxed);
            assert!(prev % 2 == 0, "Monotonic must have missed an interrupt!");
        }
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create a TIMER0-based monotonic and register the TIMER0 interrupt for it.
///
/// See [`crate::silabs::timer`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The desired tick rate of the monotonic.
#[macro_export]
macro_rules! silabs_timer0_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        /// A `Monotonic` based on the EFR32's 32-bit TIMER0 peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// `tim_clock_hz` is the EM01GRPACLK frequency feeding TIMER0.
            ///
            /// This method must be called only once.
            pub fn start(tim_clock_hz: u32) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn TIMER0() {
                    use $crate::TimerQueueBackend;
                    $crate::silabs::timer::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::silabs::timer::TimerBackend::_start(tim_clock_hz, $tick_rate_hz);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::silabs::timer::TimerBackend;
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
