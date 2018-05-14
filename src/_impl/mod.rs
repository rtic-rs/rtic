use core::mem;

pub use self::instant::Instant;
pub use self::tq::{dispatch, NotReady, TimerQueue};
pub use cortex_m::interrupt;
use cortex_m::interrupt::Nr;
pub use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::{CBP, CPUID, DCB, DWT, FPB, FPU, ITM, MPU, NVIC, SCB, SYST, TPIU};
use heapless::RingBuffer as Queue;
pub use typenum::consts::*;
pub use typenum::{Max, Maximum, Unsigned};

mod instant;
mod tq;

pub type FreeQueue<N> = Queue<u8, N, u8>;
pub type ReadyQueue<T, N> = Queue<(T, u8), N, u8>;

#[cfg(feature = "timer-queue")]
pub struct Peripherals<'a> {
    pub CBP: CBP,
    pub CPUID: CPUID,
    pub DCB: DCB,
    pub FPB: FPB,
    pub FPU: FPU,
    pub ITM: ITM,
    pub MPU: MPU,
    // pub NVIC: NVIC,
    pub SCB: &'a mut SCB,
    pub TPIU: TPIU,
}

#[cfg(not(feature = "timer-queue"))]
pub struct Peripherals {
    pub CBP: CBP,
    pub CPUID: CPUID,
    pub DCB: DCB,
    pub DWT: DWT,
    pub FPB: FPB,
    pub FPU: FPU,
    pub ITM: ITM,
    pub MPU: MPU,
    // pub NVIC: NVIC,
    pub SCB: SCB,
    pub SYST: SYST,
    pub TPIU: TPIU,
}

pub fn trigger<I>(interrupt: I)
where
    I: Nr,
{
    unsafe { mem::transmute::<(), NVIC>(()).set_pending(interrupt) }
}

pub const unsafe fn uninitialized<T>() -> T {
    #[allow(unions_with_drop_fields)]
    union U<T> {
        some: T,
        none: (),
    }

    U { none: () }.some
}

pub unsafe fn steal() -> ::cortex_m::Peripherals {
    ::cortex_m::Peripherals::steal()
}
