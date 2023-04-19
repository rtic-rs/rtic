/// No-operation function. Usually, it is just a re-export of the
/// `nop` function of the architecture crate (e.g., [`cortex_m::nop`]).
pub unsafe fn nop() {}

/// Module to enable/disable interrupts. Usually, it is just a re-export of the
/// `interrupt` module of the architecture crate (e.g., [`cortex_m::interrupt`]).
pub mod interrupt {
    /// Disables interrupts at core/HART level.
    pub unsafe fn disable() {}
    /// Enables interrupts at core/HART level.
    pub unsafe fn enable() {}
}

/// Target core peripherals struct. In [`cortex_m`] targets, this is just
/// [`cortex_m::Peripherals`]. However, [`riscv`] targets do not have such a
/// well standardized interface, and the core peripherals struct might vary.
pub struct Peripherals();

/// Trait used to represent the interrupts used by RTIC. In [`cortex_m`] targets,
/// this is just [`cortex_m::interrupt::InterruptNumber`]. However, [`riscv`] targets
/// do not have such a well standardized interface, and the target trait might vary.
pub unsafe trait InterruptNumber {}

/// Sets the given interrupt source as pending.
#[inline(always)]
pub fn pend<I>(interrupt: I)
where
    I: InterruptNumber,
{
}

/// Runs a function with a given priority mask.
#[inline(always)]
pub unsafe fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
}

/// Runs a function that takes a shared resource with a priority ceiling.
/// This function returns the return value of the target function.
#[inline(always)]
pub unsafe fn lock<F, T, R>(ptr: *mut T, ceiling: u8, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    f(&mut *ptr)
}

/// You can add any additional feature you want.
pub mod others {}
