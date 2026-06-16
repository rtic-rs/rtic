//! [`Monotonic`](rtic_time::Monotonic) implementation for ESP32's Timer Group 0, Timer 0.
//!
//! Runs at APB clock / 2. With esp-hal's default clock config (240 MHz CPU, 80 MHz APB)
//! this is 40 MHz, giving 25 ns resolution.
//!
//! Note: this uses TIMG0 Timer 0 exclusively. Do not use that timer when using monotonics.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::esp32::prelude::*;
//!
//! esp32_timg0_monotonic!(Mono);
//!
//! fn init() {
//!     # // Normally provided by esp_hal::init()
//!     # let timer = unsafe { &*esp32::TIMG0::ptr() };
//!     #
//!     // Start the monotonic
//!     Mono::start(timer);
//! }
//!
//! async fn usage() {
//!     loop {
//!         let timestamp = Mono::now();
//!         Mono::delay(100.millis()).await;
//!     }
//! }
//! ```

/// Common definitions and traits for using the ESP32 TIMG0 Timer0 monotonic.
pub mod prelude {
    pub use crate::esp32_timg0_monotonic;

    pub use crate::Monotonic;

    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

use crate::TimerQueueBackend;
use rtic_time::timer_queue::TimerQueue;

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(_timer: esp_hal::peripherals::TIMG0<'static>) {
        unsafe {
            let timg0 = &*esp32::TIMG0::ptr();
            let t = timg0.t(0);

            //configure TIMG0 Timer0 as a 64-bit up-counter
            t.config().modify(|_, w| {
                w.alarm_en()
                    .clear_bit() //disable alarm initially
                    .level_int_en()
                    .set_bit() //level-triggered interrupt
                    .edge_int_en()
                    .clear_bit()
                    .divider()
                    .bits(2) //prescaler = 2 gives 40MHz
                    .autoreload()
                    .clear_bit() //no reload on alarm
                    .increase()
                    .set_bit() //count up
                    .en()
                    .set_bit() // enable timer
            });

            //load 0 as initial counter value.
            t.loadlo().write(|w| w.load_lo().bits(0));
            t.loadhi().write(|w| w.load_hi().bits(0));
            t.load().write(|w| w.load().bits(0));

            //enable the timer interrupt in the TIMG0 interrupt-enable register
            timg0.int_ena().modify(|_, w| w.t0().set_bit());
        }

        //route the peripheral interrupt through DPORT
        esp_hal::interrupt::enable(
            esp32::Interrupt::TG0_T0_LEVEL,
            esp_hal::interrupt::Priority::Priority1,
        );

        TIMER_QUEUE.initialize(Self {});
    }
}

static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();

impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        let timg0 = unsafe { &*esp32::TIMG0::ptr() };
        let t = timg0.t(0);
        //writing UPDATE latches the current counter value into LO/HI
        t.update().write(|w| w.update().set_bit());
        let lo = t.lo().read().lo().bits() as u64;
        let hi = t.hi().read().hi().bits() as u64;
        lo | (hi << 32)
    }

    fn set_compare(instant: Self::Ticks) {
        let timg0 = unsafe { &*esp32::TIMG0::ptr() };
        let t = timg0.t(0);
        t.alarmlo()
            .write(|w| unsafe { w.alarm_lo().bits(instant as u32) });
        t.alarmhi()
            .write(|w| unsafe { w.alarm_hi().bits((instant >> 32) as u32) });
        t.config().modify(|_, w| w.alarm_en().set_bit());
    }

    fn clear_compare_flag() {
        let timg0 = unsafe { &*esp32::TIMG0::ptr() };
        // Writing 1 clears the timer-0 interrupt status.
        timg0.int_clr().write(|w| w.t0().clear_bit_by_one());
    }

    fn pend_interrupt() {
        extern "C" {
            fn TG0_T0_LEVEL();
        }
        //call the timer ISR directly in a critical section to immediately process the
        //timer queue, since there is no way to software-pend a peripheral interrupt
        critical_section::with(|_| unsafe { TG0_T0_LEVEL() });
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create an ESP32 TIMG0 Timer0 based monotonic and register the necessary interrupt for it.
///
/// See [`crate::esp32`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! esp32_timg0_monotonic {
    ($name:ident) => {
        /// A `Monotonic` based on ESP32's TIMG0 Timer0, running at APB / 2 (40 MHz).
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: esp_hal::peripherals::TIMG0<'static>) {
                #[export_name = "TG0_T0_LEVEL"]
                #[allow(non_snake_case)]
                unsafe extern "C" fn Timg0Timer0() {
                    use $crate::TimerQueueBackend;
                    $crate::esp32::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::esp32::TimerBackend::_start(timer);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::esp32::TimerBackend;
            type Instant = $crate::fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                40_000_000,
            >;
            type Duration = $crate::fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                40_000_000,
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}
