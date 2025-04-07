pub use esp32c6::{Interrupt, Peripherals};
use esp32c6::{INTERRUPT_CORE0, PLIC_MX};
pub use riscv::interrupt;
pub use riscv::register::mcause;

#[cfg(all(feature = "riscv-esp32c6", not(feature = "riscv-esp32c6-backend")))]
compile_error!("Building for the esp32c6, but 'riscv-esp32c6-backend not selected'");

#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        //if priority is 1, priority thresh should be 1
        f();
        unsafe {
            (*PLIC_MX::ptr())
                .mxint_thresh()
                .write(|w| w.cpu_mxint_thresh().bits(1));
        }
    } else {
        //read current thresh
        let initial = unsafe {
            (*PLIC_MX::ptr())
                .mxint_thresh()
                .read()
                .cpu_mxint_thresh()
                .bits()
        };
        f();
        //write back old thresh
        unsafe {
            (*PLIC_MX::ptr())
                .mxint_thresh()
                .write(|w| w.cpu_mxint_thresh().bits(initial));
        }
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
    if ceiling == (15) {
        // Turn off interrupts completely, we're at max prio
        critical_section::with(|_| f(&mut *ptr))
    } else {
        let current = unsafe {
            (*PLIC_MX::ptr())
                .mxint_thresh()
                .read()
                .cpu_mxint_thresh()
                .bits()
        };

        // esp32c6 lets interrupts with prio equal to threshold through so we up it by one
        unsafe {
            (*PLIC_MX::ptr())
                .mxint_thresh()
                .write(|w| w.cpu_mxint_thresh().bits(ceiling + 1));
        }

        let r = f(&mut *ptr);

        unsafe {
            (*PLIC_MX::ptr())
                .mxint_thresh()
                .write(|w| w.cpu_mxint_thresh().bits(current));
        }

        r
    }
}

/// Sets the given software interrupt as pending
#[inline(always)]
pub fn pend(int: Interrupt) {
    unsafe {
        let peripherals = Peripherals::steal();
        match int {
            Interrupt::FROM_CPU_INTR0 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_0()
                .write(|w| w.cpu_intr_from_cpu_0().bit(true)),
            Interrupt::FROM_CPU_INTR1 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_1()
                .write(|w| w.cpu_intr_from_cpu_1().bit(true)),
            Interrupt::FROM_CPU_INTR2 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_2()
                .write(|w| w.cpu_intr_from_cpu_2().bit(true)),
            Interrupt::FROM_CPU_INTR3 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_3()
                .write(|w| w.cpu_intr_from_cpu_3().bit(true)),
            _ => panic!("Unsupported software interrupt"), //should never happen, checked at compile time
        };
    }
}

// Sets the given software interrupt as not pending
pub fn unpend(int: Interrupt) {
    unsafe {
        let peripherals = Peripherals::steal();
        match int {
            Interrupt::FROM_CPU_INTR0 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_0()
                .write(|w| w.cpu_intr_from_cpu_0().bit(false)),
            Interrupt::FROM_CPU_INTR1 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_1()
                .write(|w| w.cpu_intr_from_cpu_1().bit(false)),
            Interrupt::FROM_CPU_INTR2 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_2()
                .write(|w| w.cpu_intr_from_cpu_2().bit(false)),
            Interrupt::FROM_CPU_INTR3 => peripherals
                .INTPRI
                .cpu_intr_from_cpu_3()
                .write(|w| w.cpu_intr_from_cpu_3().bit(false)),
            _ => panic!("Unsupported software interrupt"),
        };
    }
}

pub fn enable(int: Interrupt, prio: u8, cpu_int_id: u8) {
    unsafe {
        // Map the peripheral interrupt to a CPU interrupt:
        (INTERRUPT_CORE0::ptr() as *mut u32)
            .offset(int as isize)
            .write_volatile(cpu_int_id as u32);

        // Set the interrupt's priority:
        (*PLIC_MX::ptr())
            .mxint_pri(cpu_int_id as usize)
            .write(|w| w.bits(prio as u32));

        // Finally, enable the CPU interrupt:
        (*PLIC_MX::ptr())
            .mxint_enable()
            .modify(|r, w| w.bits((1 << cpu_int_id) | r.bits()));
    }
}
