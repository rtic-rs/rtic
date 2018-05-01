// #![deny(missing_docs)]
// #![deny(warnings)]
#![allow(warnings)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![feature(untagged_unions)]
#![feature(never_type)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm_macros;
extern crate heapless;
extern crate typenum;

mod instant;
mod node;
mod resource;
mod tq;

use core::mem;

#[doc(hidden)]
pub use cortex_m::interrupt;
use cortex_m::interrupt::Nr;
#[doc(hidden)]
pub use cortex_m::peripheral::syst::SystClkSource;
#[cfg(any(has_fpu, target_arch = "x86_64"))]
use cortex_m::peripheral::FPU;
use cortex_m::peripheral::{Peripherals, CPUID, DCB, DWT, MPU, NVIC, SCB, SYST};
#[cfg(any(armv7m, target_arch = "x86_64"))]
use cortex_m::peripheral::{CBP, FPB, ITM, TPIU};
pub use cortex_m_rtfm_macros::app;
use heapless::ring_buffer::RingBuffer;
pub use typenum::consts::*;
pub use typenum::{Max, Maximum, Unsigned};

pub use instant::Instant;
pub use node::Node;
use node::{Slot, TaggedPayload};
pub use resource::{Resource, Threshold};
pub use tq::{dispatch, TimerQueue};

pub type PayloadQueue<T, N> = RingBuffer<TaggedPayload<T>, N, u8>;
pub type SlotQueue<T, N> = RingBuffer<Slot<T>, N, u8>;
pub type Ceiling<R> = <R as Resource>::Ceiling;

pub struct Core {
    #[cfg(any(armv7m, target_arch = "x86_64"))]
    pub CBP: CBP,
    pub CPUID: CPUID,
    pub DCB: DCB,
    // pub DWT: DWT,
    #[cfg(any(armv7m, target_arch = "x86_64"))]
    pub FPB: FPB,
    #[cfg(any(has_fpu, target_arch = "x86_64"))]
    pub FPU: FPU,
    #[cfg(any(armv7m, target_arch = "x86_64"))]
    pub ITM: ITM,
    pub MPU: MPU,
    pub SCB: SCB,
    // pub SYST: SYST,
    #[cfg(any(armv7m, target_arch = "x86_64"))]
    pub TPIU: TPIU,
}

impl Core {
    pub unsafe fn steal() -> (Core, DWT, NVIC, SYST) {
        let p = Peripherals::steal();

        (
            Core {
                #[cfg(any(armv7m, target_arch = "x86_64"))]
                CBP: p.CBP,
                CPUID: p.CPUID,
                DCB: p.DCB,
                #[cfg(any(armv7m, target_arch = "x86_64"))]
                FPB: p.FPB,
                #[cfg(any(has_fpu, target_arch = "x86_64"))]
                FPU: p.FPU,
                #[cfg(any(armv7m, target_arch = "x86_64"))]
                ITM: p.ITM,
                MPU: p.MPU,
                SCB: p.SCB,
                #[cfg(any(armv7m, target_arch = "x86_64"))]
                TPIU: p.TPIU,
            },
            p.DWT,
            p.NVIC,
            p.SYST,
        )
    }
}

pub fn atomic<R, P, F>(t: &mut Threshold<P>, f: F) -> R
where
    F: FnOnce(&mut Threshold<U255>) -> R,
    P: Unsigned,
{
    unsafe {
        debug_assert!(P::to_u8() <= 255);

        if P::to_u8() < 255 {
            interrupt::disable();
            let r = f(&mut Threshold::new());
            interrupt::enable();
            r
        } else {
            f(&mut Threshold::new())
        }
    }
}

#[doc(hidden)]
pub const unsafe fn uninitialized<T>() -> T {
    #[allow(unions_with_drop_fields)]
    union U<T> {
        some: T,
        none: (),
    }

    U { none: () }.some
}

#[doc(hidden)]
pub unsafe fn set_pending<I>(interrupt: I)
where
    I: Nr,
{
    mem::transmute::<(), NVIC>(()).set_pending(interrupt)
}
