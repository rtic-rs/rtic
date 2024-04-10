pub use esp32c6::{Interrupt, Peripherals};
use esp32c6::{INTERRUPT_CORE0, INTPRI};
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
            (*INTPRI::ptr())
                .cpu_int_thresh()
                .write(|w| w.cpu_int_thresh().bits(1));
        }
    } else {
        //read current thresh
        let initial = unsafe {
            (*INTPRI::ptr())
                .cpu_int_thresh()
                .read()
                .cpu_int_thresh()
                .bits()
        };
        f();
        //write back old thresh
        unsafe {
            (*INTPRI::ptr())
                .cpu_int_thresh()
                .write(|w| w.cpu_int_thresh().bits(initial));
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
        //turn off interrupts completely, were at max prio
        let r = critical_section::with(|_| f(&mut *ptr));
        r
    } else {
        let current = unsafe {
            (*INTPRI::ptr())
                .cpu_int_thresh()
                .read()
                .cpu_int_thresh()
                .bits()
        };

        unsafe {
            (*INTPRI::ptr())
                .cpu_int_thresh()
                .write(|w| w.cpu_int_thresh().bits(ceiling + 1))
        } //esp32c6 lets interrupts with prio equal to threshold through so we up it by one
        let r = f(&mut *ptr);
        unsafe {
            (*INTPRI::ptr())
                .cpu_int_thresh()
                .write(|w| w.cpu_int_thresh().bits(current))
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
        }
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
        }
    }
}

pub fn enable(int: Interrupt, prio: u8, cpu_int_id: u8) {
    const INTERRUPT_MAP_BASE: *mut u32 =
        unsafe { core::mem::transmute::<_, *mut u32>(INTERRUPT_CORE0::ptr()) };
    let interrupt_number = int as isize;
    let cpu_interrupt_number = cpu_int_id as isize;

    unsafe {
        let intr_map_base = INTERRUPT_MAP_BASE as *mut u32;
        intr_map_base
            .offset(interrupt_number)
            .write_volatile(cpu_interrupt_number as u32);

        let intr_prio_base = (*INTPRI::ptr()).cpu_int_pri_0().as_ptr();
        intr_prio_base
            .offset(cpu_interrupt_number)
            .write_volatile(prio as u32);

        (*INTPRI::ptr())
            .cpu_int_enable()
            .modify(|r, w| w.bits((1 << cpu_interrupt_number) | r.bits()));
    }
}
