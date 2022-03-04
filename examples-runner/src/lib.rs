#![no_std]


pub use embedded_ci_pac as pac; // memory layout

#[cfg(feature = "embedded-ci")]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "qemu")]
use panic_semihosting as _;

#[cfg(feature = "embedded-ci")]
use defmt_rtt as _; // global logger
#[cfg(feature = "embedded-ci")]
use panic_probe as _;

#[cfg(feature = "embedded-ci")]
defmt::timestamp! {"{=u64}", {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[cfg(feature = "embedded-ci")]
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Generalize println for QEMU and defmt
#[cfg(feature = "embedded-ci")]
#[macro_export]
macro_rules! println {
    ($($l:tt)*) => {
        defmt::println!($($l)*);
    }
}

#[cfg(feature = "qemu")]
#[macro_export]
macro_rules! println {
    ($($l:tt)*) => {
        cortex_m_semihosting::hprintln!($($l)*).ok();
    }
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    #[cfg(feature = "qemu")]
    cortex_m_semihosting::debug::exit(cortex_m_semihosting::debug::EXIT_SUCCESS);

    loop {
        cortex_m::asm::bkpt();
    }
}
