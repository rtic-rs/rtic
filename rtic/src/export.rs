pub use bare_metal::CriticalSection;
//pub use portable_atomic as atomic;
pub use atomic_polyfill as atomic;

pub mod executor;

#[cfg(all(
    feature = "cortex-m-basepri",
    not(any(feature = "thumbv7-backend", feature = "thumbv8main-backend"))
))]
compile_error!(
    "Building for Cortex-M with basepri, but 'thumbv7-backend' or 'thumbv8main-backend' backend not selected"
);

#[cfg(all(
    feature = "cortex-m-source-masking",
    not(any(feature = "thumbv6-backend", feature = "thumbv8base-backend"))
))]
compile_error!(
    "Building for Cortex-M with source masking, but 'thumbv6-backend' or 'thumbv8base-backend' backend not selected"
);

//#[cfg(feature = "riscv-esp32c3", not(feature = "riscv32-esp32c3-backend"))]
//compile_error!(
   // "Building for the esp32c3, but 'riscv32-esp32c3-backend not selected'"
//); 

#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
pub use cortex_basepri::*;

#[cfg(any(feature = "cortex-m-basepri", feature = "rtic-uitestv7"))]
mod cortex_basepri;

#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv6"))]
pub use cortex_source_mask::*;

#[cfg(any(feature = "cortex-m-source-masking", feature = "rtic-uitestv6"))]
mod cortex_source_mask;

#[cfg(feature = "cortex-m")]
pub use cortex_m::{interrupt::InterruptNumber, peripheral::NVIC};

#[cfg(feature = "cortex-m")]
pub use cortex_m::{interrupt::InterruptNumber, peripheral::NVIC};

/// Sets the given `interrupt` as pending
///
/// This is a convenience function around
/// [`NVIC::pend`](../cortex_m/peripheral/struct.NVIC.html#method.pend)
#[cfg(feature = "cortex-m")]
pub fn pend<I>(interrupt: I)
where
    I: InterruptNumber,
{
    NVIC::pend(interrupt);
}


#[cfg(feature = "riscv-esp32c3")]
mod riscv_esp32c3;
#[cfg(feature = "riscv-esp32c3")]
pub use riscv_esp32c3::*;

///I think all of these "pends" and "unpends"  should be moved to /export/<device>.rs
#[cfg(feature = "riscv-esp32c3")]
/// Sets the given software interrupt as pending
pub fn pend(int: Interrupt){
    unsafe{
    let peripherals = Peripherals::steal();
        match int{
            Interrupt::FROM_CPU_INTR0 => peripherals.SYSTEM.cpu_intr_from_cpu_0.write(|w|w.cpu_intr_from_cpu_0().bit(true)),
            Interrupt::FROM_CPU_INTR1 => peripherals.SYSTEM.cpu_intr_from_cpu_1.write(|w|w.cpu_intr_from_cpu_1().bit(true)),
            Interrupt::FROM_CPU_INTR2 => peripherals.SYSTEM.cpu_intr_from_cpu_2.write(|w|w.cpu_intr_from_cpu_2().bit(true)),
            Interrupt::FROM_CPU_INTR3 => peripherals.SYSTEM.cpu_intr_from_cpu_3.write(|w|w.cpu_intr_from_cpu_3().bit(true)),      
            _ => panic!("Unsupported software interrupt"), //unsupported sw interrupt provided, panic for now. Eventually we can check this at compile time also.
        }
    }   
}


#[cfg(feature = "riscv-esp32c3")]
pub fn unpend(int: Interrupt){
    unsafe{
        let peripherals = Peripherals::steal();
            match int{
                Interrupt::FROM_CPU_INTR0 => peripherals.SYSTEM.cpu_intr_from_cpu_0.write(|w|w.cpu_intr_from_cpu_0().bit(false)),
                Interrupt::FROM_CPU_INTR1 => peripherals.SYSTEM.cpu_intr_from_cpu_1.write(|w|w.cpu_intr_from_cpu_1().bit(false)),
                Interrupt::FROM_CPU_INTR2 => peripherals.SYSTEM.cpu_intr_from_cpu_2.write(|w|w.cpu_intr_from_cpu_2().bit(false)),
                Interrupt::FROM_CPU_INTR3 => peripherals.SYSTEM.cpu_intr_from_cpu_3.write(|w|w.cpu_intr_from_cpu_3().bit(false)),      
                _ => panic!("Unsupported software interrupt"), //this should realistically never happen, since tasks that call unpend must call pend first.
            }
        }   
}

/// Priority conversion, takes logical priorities 1..=N and converts it to NVIC priority.
#[cfg(feature = "cortex-m")]
#[inline]
#[must_use]
pub const fn cortex_logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}

#[inline(always)]
pub fn assert_send<T>()
where
    T: Send,
{
}

#[inline(always)]
pub fn assert_sync<T>()
where
    T: Sync,
{
}
