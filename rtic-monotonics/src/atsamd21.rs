//! [`Monotonic`](rtic_time::Monotonic) implementation for the TC4/5 timers.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::atsamd21::prelude::*;
//! atsamd21_tc4_tc5_monotonic!(Mono);
//!
//! fn init(mut device: pac::Peripherals) {
//!     let mut clocks = GenericClockController::with_internal_32kosc(
//!         device.gclk,
//!         &mut device.pm,
//!         &mut device.sysctrl,
//!         &mut device.nvmctrl,
//!     );
//!     let gclk0 = clocks.gclk0();
//!     let _tc4_tc5_clk = clocks.tc4_tc5(&gclk0).unwrap();
//!     Mono::start(device.tc4, device.tc5, &mut device.pm);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          let timestamp = Mono::now();
//!          Mono::delay(100.millis()).await;
//!     }
//! }
//!
//! // FIXME: the interrupt handler is not working, but re-implementing it in a RTIC task does
//! // Comment the interrupt handler `unsafe extern "C" fn TC4()` and add the following RTIC task
//! #[task(binds = TC4)]
//! fn tc4(_cx: tc4::Context) {
//!     use rtic_time::timer_queue::TimerQueueBackend;
//!     unsafe { Tc4Tc5Backend::timer_queue().on_monotonic_interrupt() };
//! }
//! ```

/// Common definitions and traits for using the ATSAMD21 TC4/5 monotonic
pub mod prelude {
    pub use crate::atsamd21_tc4_tc5_monotonic;
    pub use crate::Monotonic;
    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

use atsamd21g::Pm;

#[cfg(feature = "atsamd21g")]
use atsamd21g as pac;

use portable_atomic::{AtomicU32, Ordering};
use rtic_time::{
    half_period_counter::calculate_now,
    timer_queue::{TimerQueue, TimerQueueBackend},
};

static HALF_PERIOD_COUNT: AtomicU32 = AtomicU32::new(0);
static TIMER_QUEUE: TimerQueue<Tc4Tc5Backend> = TimerQueue::new();

/// TC4/5 based [`TimerQueueBackend`].
pub struct Tc4Tc5Backend;

impl Tc4Tc5Backend {
    #[inline]
    fn register() -> &'static pac::tc4::Count32 {
        unsafe { pac::Tc4::ptr().as_ref().unwrap().count32() }
    }

    #[inline]
    fn sync() {
        while Self::register().status().read().syncbusy().bit_is_set() {}
    }

    /// Starts the clock.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the [`atsamd21_tc4_tc5_monotonic`] macro instead.
    pub fn _start(tc4: pac::Tc4, _tc5: pac::Tc5, pm: &mut Pm) {
        let tc4 = &mut tc4.count32();

        // Enable the TC4 clock
        pm.apbcmask().modify(|_, w| w.tc4_().set_bit());

        // Disable the peripheral while we reconfigure it
        tc4.ctrla().modify(|_, w| w.enable().clear_bit());
        Self::sync();

        // Reset the peripheral
        tc4.ctrla().write(|w| w.swrst().set_bit());
        Self::sync();

        // Set the counter to 32 bits
        tc4.ctrla().write(|w| w.mode().count32());
        Self::sync();

        // Reset the counter to 0
        tc4.count().reset();

        // Prepare the half-period counter and timer queue
        HALF_PERIOD_COUNT.store(0, Ordering::SeqCst);
        TIMER_QUEUE.initialize(Self);

        // Continuously update the counter register withotu having to sync
        tc4.readreq()
            .write(|w| unsafe { w.rcont().set_bit().addr().bits(0x10) });

        // We extend the 32 bit counter to 63 bits using half-period counting.
        // On overflow and half period, we increment the half-period counter.
        // We use comparator 0 for user timing, comparator 1 for half-period counting.
        tc4.intenset().write(|w| w.ovf().set_bit().mc1().set_bit());
        tc4.cc(0).write(|w| unsafe { w.cc().bits(0x0) }); // Used for timer queue, interrupt disabled by default
        tc4.cc(1).write(|w| unsafe { w.cc().bits(0x8000_0000) }); // Half-period value

        // Enable the timer
        tc4.ctrla().modify(|_, w| w.enable().set_bit());
        Self::sync();
    }
}

impl TimerQueueBackend for Tc4Tc5Backend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        calculate_now(
            || HALF_PERIOD_COUNT.load(Ordering::Relaxed),
            || Self::register().count().read().bits(),
        )
    }

    fn clear_compare_flag() {
        let reg = Self::register();
        let intflag = reg.intflag().read();

        if intflag.mc0().bit_is_set() {
            reg.intflag().write(|w| w.mc0().set_bit());
        }
    }

    fn on_interrupt() {
        let reg = Self::register();
        let intflag = reg.intflag().read();

        if intflag.ovf().bit_is_set() {
            reg.intflag().write(|w| w.ovf().set_bit());
            let prev = HALF_PERIOD_COUNT.fetch_add(1, Ordering::Relaxed);
            assert!(prev % 2 == 1, "Monotonic must have skipped an interrupt!");
        }
        if intflag.mc1().bit_is_set() {
            reg.intflag().write(|w| w.mc1().set_bit());
            let prev = HALF_PERIOD_COUNT.fetch_add(1, Ordering::Relaxed);
            assert!(prev % 2 == 0, "Monotonic must have skipped an interrupt!");
        }
    }

    fn set_compare(instant: Self::Ticks) {
        let now = Self::now();

        // Since the timer may overflow based on the requested compare val, we check how many ticks are left.
        // `wrapping_sub` takes care of the u64 integer overflow special case.
        let val = if instant.wrapping_sub(now) <= (u32::MAX as u64) {
            instant
        } else {
            // In the past or will overflow
            0
        };

        // Set the compare value and enable the interrupt
        Self::register()
            .cc(0)
            .write(|w| unsafe { w.bits(val as u32) });
        Self::register().intenset().write(|w| w.mc0().set_bit());
    }

    fn pend_interrupt() {
        pac::NVIC::pend(pac::Interrupt::TC4);
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! atsamd21_tc4_tc5_monotonic {
    ($name:ident) => {
        /// A `Monotonic` based on the TC4/5 peripherals.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(tc4: pac::Tc4, tc5: pac::Tc5, pm: &mut pac::Pm) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn TC4() {
                    defmt::info!("test");
                    use $crate::TimerQueueBackend;
                    $crate::atsamd21::Tc4Tc5Backend::timer_queue().on_monotonic_interrupt();
                }

                $crate::atsamd21::Tc4Tc5Backend::_start(tc4, tc5, pm);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::atsamd21::Tc4Tc5Backend;
            type Instant = $crate::fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                48_000_000,
            >;
            type Duration = $crate::fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                48_000_000,
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}
