//! [`Monotonic`](rtic_time::Monotonic) implementation for ESP32-C6's SYSTIMER.
//!
//! Always runs at a fixed rate of 16 MHz.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::esp32c6::prelude::*;
//!
//! esp32c6_systimer_monotonic!(Mono);
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let timer = unsafe { esp32c6::Peripherals::steal() }.SYSTIMER;
//!     #
//!     // Start the monotonic
//!     Mono::start(timer);
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

/// Common definitions and traits for using the ESP32-C6 timer monotonic
pub mod prelude {
    pub use crate::esp32c6_systimer_monotonic;

    pub use crate::Monotonic;

    pub use fugit::{self, ExtU64, ExtU64Ceil};
}
use crate::TimerQueueBackend;
use esp32c6::{INTERRUPT_CORE0, PLIC_MX, SYSTIMER};
use rtic_time::timer_queue::TimerQueue;

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(timer: SYSTIMER) {
        let interrupt_number = 57 as isize;
        let cpu_interrupt_number = 31 as isize;

        unsafe {
            (INTERRUPT_CORE0::ptr() as *mut u32)
                .offset(interrupt_number as isize)
                .write_volatile(cpu_interrupt_number as u32);

            // Set the interrupt's priority:
            (*PLIC_MX::ptr())
                .mxint_pri(cpu_interrupt_number as usize)
                .write(|w| w.bits(15 as u32));

            // Finally, enable the CPU interrupt:
            (*PLIC_MX::ptr())
                .mxint_enable()
                .modify(|r, w| w.bits((1 << cpu_interrupt_number) | r.bits()));
        }

        timer.conf().write(|w| w.timer_unit0_work_en().set_bit());
        timer
            .conf()
            .write(|w| w.timer_unit1_core0_stall_en().clear_bit());

        TIMER_QUEUE.initialize(Self {})
    }
}

static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();
use esp32c6;
impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;
    fn now() -> Self::Ticks {
        let peripherals = unsafe { esp32c6::Peripherals::steal() };
        peripherals
            .SYSTIMER
            .unit0_op()
            .write(|w| w.update().set_bit());
        // this must be polled until value is valid
        while peripherals.SYSTIMER.unit0_op().read().value_valid() == false {}
        let instant: u64 = (peripherals.SYSTIMER.unit_value(0).lo().read().bits() as u64)
            | ((peripherals.SYSTIMER.unit_value(0).hi().read().bits() as u64) << 32);
        instant
    }

    fn set_compare(instant: Self::Ticks) {
        let systimer = unsafe { esp32c6::Peripherals::steal() }.SYSTIMER;
        systimer
            .target0_conf()
            .write(|w| w.timer_unit_sel().set_bit());
        systimer
            .target0_conf()
            .write(|w| w.period_mode().clear_bit());
        systimer
            .trgt(0)
            .lo()
            .write(|w| unsafe { w.bits((instant & 0xFFFFFFFF).try_into().unwrap()) });
        systimer
            .trgt(0)
            .hi()
            .write(|w| unsafe { w.bits((instant >> 32).try_into().unwrap()) });
        systimer.comp0_load().write(|w| w.load().set_bit()); //sync period to comp register
        systimer.conf().write(|w| w.target0_work_en().set_bit());
        systimer.int_ena().write(|w| w.target0().set_bit());
    }

    fn clear_compare_flag() {
        unsafe { esp32c6::Peripherals::steal() }
            .SYSTIMER
            .int_clr()
            .write(|w| w.target0().bit(true));
    }

    fn pend_interrupt() {
        extern "C" {
            fn interrupt31();
        }
        //run the timer interrupt handler in a critical section to emulate a max priority
        //interrupt.
        //since there is no hardware support for pending a timer interrupt.
        riscv::interrupt::disable();
        unsafe { interrupt31() };
        unsafe { riscv::interrupt::enable() };
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create an ESP32-C6 SysTimer based monotonic and register the necessary interrupt for it.
///
/// See [`crate::esp32c6`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! esp32c6_systimer_monotonic {
    ($name:ident) => {
        /// A `Monotonic` based on the ESP32-C6 SysTimer peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: esp32c6::SYSTIMER) {
                #[export_name = "interrupt31"]
                #[allow(non_snake_case)]
                unsafe extern "C" fn Systimer() {
                    use $crate::TimerQueueBackend;
                    $crate::esp32c6::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::esp32c6::TimerBackend::_start(timer);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::esp32c6::TimerBackend;
            type Instant = $crate::fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                16_000_000,
            >;
            type Duration = $crate::fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                16_000_000,
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}
