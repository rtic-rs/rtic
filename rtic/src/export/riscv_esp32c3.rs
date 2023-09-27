use esp32c3::INTERRUPT_CORE0; //priority threshold control
pub use esp32c3::{Interrupt, Peripherals};
pub use riscv::{interrupt, register::mcause}; //low level interrupt enable/disable

#[cfg(all(feature = "riscv-esp32c3", not(feature = "riscv-esp32c3-backend")))]
compile_error!("Building for the esp32c3, but 'riscv-esp32c3-backend not selected'");

#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        //if priority is 1, priority thresh should be 1
        f();
        unsafe {
            (*INTERRUPT_CORE0::ptr())
                .cpu_int_thresh
                .write(|w| w.cpu_int_thresh().bits(1));
        }
    } else {
        //read current thresh
        let initial = unsafe {
            (*INTERRUPT_CORE0::ptr())
                .cpu_int_thresh
                .read()
                .cpu_int_thresh()
                .bits()
        };
        f();
        //write back old thresh
        unsafe {
            (*INTERRUPT_CORE0::ptr())
                .cpu_int_thresh
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
            (*INTERRUPT_CORE0::ptr())
                .cpu_int_thresh
                .read()
                .cpu_int_thresh()
                .bits()
        };

        unsafe {
            (*INTERRUPT_CORE0::ptr())
                .cpu_int_thresh
                .write(|w| w.cpu_int_thresh().bits(ceiling + 1))
        } //esp32c3 lets interrupts with prio equal to threshold through so we up it by one
        let r = f(&mut *ptr);
        unsafe {
            (*INTERRUPT_CORE0::ptr())
                .cpu_int_thresh
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
                .SYSTEM
                .cpu_intr_from_cpu_0
                .write(|w| w.cpu_intr_from_cpu_0().bit(true)),
            Interrupt::FROM_CPU_INTR1 => peripherals
                .SYSTEM
                .cpu_intr_from_cpu_1
                .write(|w| w.cpu_intr_from_cpu_1().bit(true)),
            Interrupt::FROM_CPU_INTR2 => peripherals
                .SYSTEM
                .cpu_intr_from_cpu_2
                .write(|w| w.cpu_intr_from_cpu_2().bit(true)),
            Interrupt::FROM_CPU_INTR3 => peripherals
                .SYSTEM
                .cpu_intr_from_cpu_3
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
                .SYSTEM
                .cpu_intr_from_cpu_0
                .write(|w| w.cpu_intr_from_cpu_0().bit(false)),
            Interrupt::FROM_CPU_INTR1 => peripherals
                .SYSTEM
                .cpu_intr_from_cpu_1
                .write(|w| w.cpu_intr_from_cpu_1().bit(false)),
            Interrupt::FROM_CPU_INTR2 => peripherals
                .SYSTEM
                .cpu_intr_from_cpu_2
                .write(|w| w.cpu_intr_from_cpu_2().bit(false)),
            Interrupt::FROM_CPU_INTR3 => peripherals
                .SYSTEM
                .cpu_intr_from_cpu_3
                .write(|w| w.cpu_intr_from_cpu_3().bit(false)),
            _ => panic!("Unsupported software interrupt"),
        }
    }
}

pub fn enable(int: Interrupt, prio: u8, cpu_int_id: u8) {
    const INTERRUPT_MAP_BASE: u32 = 0x600c2000; //this isn't exposed properly in the PAC,
                                                //should maybe figure out a workaround that
                                                //doesnt involve raw pointers.
                                                //Again, this is how they do it in the HAL
                                                //but i'm really not a fan.
    let interrupt_number = int as isize;
    let cpu_interrupt_number = cpu_int_id as isize;

    unsafe {
        let intr_map_base = INTERRUPT_MAP_BASE as *mut u32;
        intr_map_base
            .offset(interrupt_number)
            .write_volatile(cpu_interrupt_number as u32);
        //map peripheral interrupt to CPU interrupt
        (*INTERRUPT_CORE0::ptr())
            .cpu_int_enable
            .modify(|r, w| w.bits((1 << cpu_interrupt_number) | r.bits())); //enable the CPU interupt.
        let intr = INTERRUPT_CORE0::ptr();
        let intr_prio_base = (*intr).cpu_int_pri_0.as_ptr();

        intr_prio_base
            .offset(cpu_interrupt_number)
            .write_volatile(prio as u32);
    }
}
