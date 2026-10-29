//! [`Monotonic`](rtic_time::Monotonic) implementation for ESP32-C5's SYSTIMER.
//!
//! Always runs at a fixed rate of 16 MHz.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::esp32c5::prelude::*;
//!
//! esp32c5_systimer_monotonic!(Mono);
//!
//! fn init() {
//!     # // use SYSTIMER singleton from initialized esp_hal
//!     let peripherals = esp_hal::init(Config::default());
//!     // Start the monotonic
//!     Mono::start(peripherals.SYSTIMER);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          let timestamp = Mono::now();
//!          Mono::delay(100.millis()).await;
//!     }
//! }
//! ```

/// Common definitions and traits for using the ESP32-C5 timer monotonic
pub mod prelude {
    pub use crate::esp32c5_systimer_monotonic;

    pub use crate::{Monotonic, Timebase};

    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

use crate::TimerQueueBackend;
use esp_hal::{
    interrupt::{DirectBindableCpuInterrupt, Priority},
    peripherals::{Interrupt, SYSTIMER},
    system::Cpu,
    timer::systimer::{SystemTimer, Unit::Unit0, UnitConfig},
};

use rtic_time::timer_queue::TimerQueue;

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(_timer: SYSTIMER) {
        unsafe extern "C" {
            fn _start___RticMonotonicInterrupt_trap();
        }
        esp_hal::interrupt::enable_direct(
            Interrupt::SYSTIMER_TARGET0,
            Priority::max(),
            DirectBindableCpuInterrupt::Interrupt23,
            _start___RticMonotonicInterrupt_trap,
        );

        unsafe {
            SystemTimer::configure_unit(Unit0, UnitConfig::DisabledIfCpuIsStalled(Cpu::ProCpu))
        };

        TIMER_QUEUE.initialize(Self {})
    }
}

static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();

impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;
    fn now() -> Self::Ticks {
        SystemTimer::unit_value(Unit0)
    }

    fn set_compare(instant: Self::Ticks) {
        let systimer = esp_hal::peripherals::SYSTIMER::regs();
        // source=Unit0, mode=Target
        systimer.target0_conf().modify(|_, w| {
            w.timer_unit_sel().clear_bit();
            w.period_mode().clear_bit()
        });
        // set target value
        {
            let target = systimer.trgt(0);
            target.hi().write(|w| w.hi().set((instant >> 32) as u32));
            target
                .lo()
                .write(|w| w.lo().set((instant & 0xFFFF_FFFF) as u32));
            let comp_load = systimer.comp_load(0);
            comp_load.write(|w| w.load().set_bit());
        }
        //enable compare
        systimer.conf().modify(|_, w| w.target0_work_en().set_bit());
        //enable interrupt trigger
        systimer.int_ena().modify(|_, w| w.target0().set_bit());
    }

    fn clear_compare_flag() {
        esp_hal::peripherals::SYSTIMER::regs()
            .int_clr()
            .write(|w| w.target0().bit(true));
    }

    fn pend_interrupt() {
        extern "C" {
            fn __RticMonotonicInterrupt();
        }
        //run the timer interrupt handler in a critical section to emulate a max priority
        //interrupt.
        //since there is no hardware support for pending a timer interrupt.
        riscv::interrupt::disable();
        unsafe { __RticMonotonicInterrupt() };
        unsafe { riscv::interrupt::enable() };
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create an ESP32-C5 SysTimer based monotonic and register the necessary interrupt for it.
///
/// See [`crate::esp32c5`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! esp32c5_systimer_monotonic {
    ($name:ident) => {
        /// A `Monotonic` based on the ESP32-C5 SysTimer peripheral.
        pub struct $name;

        #[derive(Clone, Copy)]
        struct __RticMonotonicInterrupt;

        unsafe impl riscv_rt::InterruptNumber for __RticMonotonicInterrupt {
            const MAX_INTERRUPT_NUMBER: usize = 47;

            fn number(self) -> usize {
                esp_hal::interrupt::DirectBindableCpuInterrupt::Interrupt23 as _
            }

            fn from_number(_value: usize) -> riscv_rt::result::Result<Self> {
                Ok(Self)
            }
        }
        unsafe impl riscv_rt::CoreInterruptNumber for __RticMonotonicInterrupt {}

        #[riscv_rt::core_interrupt(__RticMonotonicInterrupt)]
        #[esp_hal::ram]
        #[allow(non_snake_case)]
        unsafe extern "C" fn Systimer() {
            use $crate::TimerQueueBackend;
            $crate::esp32c5::TimerBackend::timer_queue().on_monotonic_interrupt();
        }
        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: esp_hal::peripherals::SYSTIMER) {
                $crate::esp32c5::TimerBackend::_start(timer);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::esp32c5::TimerBackend;
            type Instant = $crate::fugit::MonotonicTimerInstant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                16_000_000,
            >;
            type Duration = $crate::fugit::TimerDuration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                16_000_000,
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}
