//! [`Monotonic`](rtic_time::Monotonic) implementation for ESP32C3's SYSTIMER.
//!
//! Always runs at a fixed rate of 16 MHz.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::esp32c3::prelude::*;
//!
//! esp32c3_systimer_monotonic!(Mono);
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let timer = unsafe { esp32c3::Peripherals::steal() }.SYSTIMER;
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

/// Common definitions and traits for using the RP2040 timer monotonic
pub mod prelude {
    pub use crate::esp32c3_systimer_monotonic;

    pub use crate::Monotonic;

    pub use fugit::{self, ExtU64, ExtU64Ceil};
}
use crate::TimerQueueBackend;
use esp32c3::{INTERRUPT_CORE0, SYSTIMER};
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
        const INTERRUPT_MAP_BASE: u32 = 0x600c2000;
        let interrupt_number = 37 as isize;
        let cpu_interrupt_number = 31 as isize;
        unsafe {
            let intr_map_base = INTERRUPT_MAP_BASE as *mut u32;
            intr_map_base
                .offset(interrupt_number)
                .write_volatile(cpu_interrupt_number as u32);
            //map peripheral interrupt to CPU interrupt
            (*INTERRUPT_CORE0::ptr())
                .cpu_int_enable()
                .modify(|r, w| w.bits((1 << cpu_interrupt_number) | r.bits())); //enable the CPU interupt.
            let intr = INTERRUPT_CORE0::ptr();
            let intr_prio_base = (*intr).cpu_int_pri_0().as_ptr();

            intr_prio_base
                .offset(cpu_interrupt_number)
                .write_volatile(15 as u32);
        }
        timer.conf().write(|w| w.timer_unit0_work_en().set_bit());
        timer
            .conf()
            .write(|w| w.timer_unit1_core0_stall_en().clear_bit());
        TIMER_QUEUE.initialize(Self {})
    }
}

static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();
use esp32c3;
impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;
    fn now() -> Self::Ticks {
        let peripherals = unsafe { esp32c3::Peripherals::steal() };
        peripherals
            .SYSTIMER
            .unit0_op()
            .write(|w| w.timer_unit0_update().set_bit());
        // this must be polled until value is valid
        while {
            peripherals
                .SYSTIMER
                .unit0_op()
                .read()
                .timer_unit0_value_valid()
                == false
        } {}
        let instant: u64 = (peripherals.SYSTIMER.unit0_value_lo().read().bits() as u64)
            | ((peripherals.SYSTIMER.unit0_value_hi().read().bits() as u64) << 32);
        instant
    }

    fn set_compare(instant: Self::Ticks) {
        let systimer = unsafe { esp32c3::Peripherals::steal() }.SYSTIMER;
        systimer
            .target0_conf()
            .write(|w| w.target0_timer_unit_sel().set_bit());
        systimer
            .target0_conf()
            .write(|w| w.target0_period_mode().clear_bit());
        systimer
            .target0_lo()
            .write(|w| unsafe { w.bits((instant & 0xFFFFFFFF).try_into().unwrap()) });
        systimer
            .target0_hi()
            .write(|w| unsafe { w.bits((instant >> 32).try_into().unwrap()) });
        systimer
            .comp0_load()
            .write(|w| w.timer_comp0_load().set_bit()); //sync period to comp register
        systimer.conf().write(|w| w.target0_work_en().set_bit());
        systimer.int_ena().write(|w| w.target0().set_bit());
    }

    fn clear_compare_flag() {
        unsafe { esp32c3::Peripherals::steal() }
            .SYSTIMER
            .int_clr()
            .write(|w| w.target0().bit(true));
    }

    fn pend_interrupt() {
        extern "C" {
            fn cpu_int_31_handler();
        }
        //run the timer interrupt handler in a critical section to emulate a max priority
        //interrupt.
        //since there is no hardware support for pending a timer interrupt.
        riscv::interrupt::disable();
        unsafe { cpu_int_31_handler() };
        unsafe { riscv::interrupt::enable() };
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create an ESP32-C3 SysTimer based monotonic and register the necessary interrupt for it.
///
/// See [`crate::esp32c3`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! esp32c3_systimer_monotonic {
    ($name:ident) => {
        /// A `Monotonic` based on the ESP32-C3 SysTimer peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: esp32c3::SYSTIMER) {
                #[export_name = "cpu_int_31_handler"]
                #[allow(non_snake_case)]
                unsafe extern "C" fn Systimer() {
                    use $crate::TimerQueueBackend;
                    $crate::esp32c3::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::esp32c3::TimerBackend::_start(timer);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::esp32c3::TimerBackend;
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
