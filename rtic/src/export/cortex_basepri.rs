use super::cortex_logical2hw;
use cortex_m::register::{basepri, basepri_max};
pub use cortex_m::{
    Peripherals,
    asm::wfi,
    interrupt,
    peripheral::{DWT, SCB, SYST, scb::SystemHandler},
};

#[cfg(not(any(feature = "thumbv7-backend", feature = "thumbv8main-backend")))]
compile_error!(
    "Building for Cortex-M with basepri, but 'thumbv7-backend' or 'thumbv8main-backend' backend not selected"
);

#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        // If the priority of this interrupt is `1` then BASEPRI can only be `0`
        f();
        unsafe { basepri::write(0) }
    } else {
        let initial = basepri::read();
        f();
        unsafe { basepri::write(initial) }
    }
}

/// Lock implementation using BASEPRI and global Critical Section (CS)
///
/// # Safety
///
/// The system ceiling is raised from current to ceiling
/// by either
/// - raising the BASEPRI to the ceiling value, or
/// - disable all interrupts in case we want to
///   mask interrupts with maximum priority
///
/// Dereferencing a raw pointer inside CS
///
/// The priority.set/priority.get can safely be outside the CS
/// as being a context local cell (not affected by preemptions).
/// It is merely used in order to omit masking in case current
/// priority is current priority >= ceiling.
///
/// Lock Efficiency:
/// Experiments validate (sub)-zero cost for CS implementation
/// (Sub)-zero as:
/// - Either zero OH (lock optimized out), or
/// - Amounting to an optimal assembly implementation
///   - The BASEPRI value is folded to a constant at compile time
///   - CS entry, single assembly instruction to write BASEPRI
///   - CS exit, single assembly instruction to write BASEPRI
///   - priority.set/get optimized out (their effect not)
/// - On par or better than any handwritten implementation of SRP
///
/// Limitations:
/// The current implementation reads/writes BASEPRI once
/// even in some edge cases where this may be omitted.
/// Total OH of per task is max 2 clock cycles, negligible in practice
/// but can in theory be fixed.
///
#[inline(always)]
pub unsafe fn lock<T, R>(
    ptr: *mut T,
    ceiling: u8,
    nvic_prio_bits: u8,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    unsafe {
        if ceiling == (1 << nvic_prio_bits) {
            critical_section::with(|_| f(&mut *ptr))
        } else {
            let current = basepri::read();
            basepri_max::write(cortex_logical2hw(ceiling, nvic_prio_bits));
            let r = f(&mut *ptr);
            basepri::write(current);
            r
        }
    }
}
