pub use esp_hal::peripherals::Interrupt;
pub use esp_hal::peripherals::Peripherals;
pub use riscv::interrupt;
pub use riscv::register::mcause;
pub use riscv_rt;

use esp_hal::interrupt::DirectBindableCpuInterrupt;
use esp_hal::interrupt::{Priority, software::SoftwareInterrupt};

mod mintthresh {
    riscv::read_csr_as_usize!(0x347);
    riscv::write_csr_as_usize!(0x347);
}

#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    unsafe {
        let mcause = mcause::read();

        if priority == 1 {
            //if priority is 1, priority thresh should be 0
            f();

            mintthresh::write(0);
        } else {
            //read current thresh
            let initial = mintthresh::read();

            if priority != Priority::max() as u8 {
                riscv::interrupt::nested(f);
            } else {
                f();
            }

            //write back old thresh
            mintthresh::write(initial);
        }

        // reused esp_hal handler code :
        // https://github.com/esp-rs/esp-hal/blob/fad4e4490c261f52c627a537ab81360b7cdd94e5/esp-hal/src/interrupt/riscv.rs#L647-L707
        // it is mandatory to restore `mcause` register
        // since it contains the former CPU priority. When executing `mret`,
        // the hardware will restore the former threshold, from `mcause` to
        // `mintstatus` CSR
        core::arch::asm!("csrw 0x342, {}", in(reg)  mcause.bits());
    }
}

/// Lock implementation using threshold and global Critical Section (CS)
///
/// # Safety
///
/// The system ceiling is raised from current to ceiling
/// by either
/// - raising the threshold to the ceiling value, or
/// - disable all interrupts in case we want to
///   mask interrupts with maximum priority
///
/// Dereferencing a raw pointer inside CS
///
/// The priority.set/priority.get can safely be outside the CS
/// as being a context local cell (not affected by preemptions).
/// It is merely used in order to omit masking in case current
/// priority is current priority >= ceiling.
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    unsafe {
        if ceiling == Priority::max() as _ {
            // Turn off interrupts completely, we're at max prio
            critical_section::with(|_| f(&mut *ptr))
        } else {
            let current = mintthresh::read();

            mintthresh::write(ceiling as _);
            let r = f(&mut *ptr);

            mintthresh::write(current);

            r
        }
    }
}

/// Sets the given software interrupt as pending
#[inline(always)]
pub fn pend(int: Interrupt) {
    unsafe {
        let peripherals = Peripherals::steal();

        match int {
            Interrupt::FROM_CPU_INTR0 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR0).raise(),
            Interrupt::FROM_CPU_INTR1 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR1).raise(),
            Interrupt::FROM_CPU_INTR2 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR2).raise(),
            Interrupt::FROM_CPU_INTR3 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR3).raise(),
            _ => panic!("Unsupported software interrupt"), //should never happen, checked at compile time
        };
    }
}

// Sets the given software interrupt as not pending
#[inline(always)]
pub fn unpend(int: Interrupt) {
    unsafe {
        let peripherals = Peripherals::steal();
        match int {
            Interrupt::FROM_CPU_INTR0 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR0).reset(),
            Interrupt::FROM_CPU_INTR1 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR1).reset(),
            Interrupt::FROM_CPU_INTR2 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR2).reset(),
            Interrupt::FROM_CPU_INTR3 => SoftwareInterrupt::new(peripherals.FROM_CPU_INTR3).reset(),
            _ => panic!("Unsupported software interrupt"),
        };
    }
}

#[inline(always)]
pub fn enable(
    int: Interrupt,
    prio: Priority,
    cpu_int: DirectBindableCpuInterrupt,
    handler: unsafe extern "C" fn(),
) {
    esp_hal::interrupt::enable_direct(int, prio, cpu_int, handler);
}
