#![feature(asm)]
#![feature(const_fn)]
#![feature(optin_builtin_traits)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm_macros;
extern crate static_ref;

use core::cell::UnsafeCell;

pub use cortex_m_rtfm_macros::rtfm;
pub use cortex_m::asm::{bkpt, wfi};
pub use cortex_m::interrupt::CriticalSection;
pub use cortex_m::interrupt::free as atomic;
pub use static_ref::Static;
use cortex_m::interrupt::Nr;
#[cfg(not(armv6m))]
use cortex_m::register::{basepri_max, basepri};

#[cfg(not(armv6m))]
macro_rules! barrier {
    () => {
        asm!("" ::: "memory" : "volatile");
    }
}

#[inline(always)]
unsafe fn claim<T, U, R, F, G>(
    data: T,
    ceiling: u8,
    nvic_prio_bits: u8,
    t: &mut Threshold,
    f: F,
    g: G,
) -> R
where
    F: FnOnce(U, &mut Threshold) -> R,
    G: FnOnce(T) -> U,
{
    let max_priority = 1 << nvic_prio_bits;
    if ceiling > t.0 {
        match () {
            #[cfg(armv6m)]
            () => {
                atomic(|_| f(g(data), &mut Threshold::new(max_priority)))
            }
            #[cfg(not(armv6m))]
            () => {
                if ceiling == max_priority {
                    atomic(|_| f(g(data), &mut Threshold::new(max_priority)))
                } else {
                    let old = basepri::read();
                    let hw = (max_priority - ceiling) << (8 - nvic_prio_bits);
                    basepri_max::write(hw);
                    barrier!();
                    let ret = f(g(data), &mut Threshold(ceiling));
                    barrier!();
                    basepri::write(old);
                    ret
                }
            }
        }
    } else {
        f(g(data), t)
    }
}

pub struct Peripheral<P>
where
    P: 'static,
{
    // FIXME(rustc/LLVM bug?) storing the ceiling in the resource de-optimizes
    // claims (the ceiling value gets loaded at runtime rather than inlined)
    // ceiling: u8,
    peripheral: cortex_m::peripheral::Peripheral<P>,
}

impl<P> Peripheral<P> {
    pub const fn new(peripheral: cortex_m::peripheral::Peripheral<P>) -> Self {
        Peripheral { peripheral }
    }

    #[inline(always)]
    pub unsafe fn claim<R, F>(
        &'static self,
        ceiling: u8,
        nvic_prio_bits: u8,
        t: &mut Threshold,
        f: F,
    ) -> R
    where
        F: FnOnce(&P, &mut Threshold) -> R,
    {
        claim(
            &self.peripheral,
            ceiling,
            nvic_prio_bits,
            t,
            f,
            |peripheral| &*peripheral.get(),
        )
    }

    pub fn get(&self) -> *mut P {
        self.peripheral.get()
    }
}

unsafe impl<P> Sync for Peripheral<P>
where
    P: Send,
{
}

pub struct Resource<T> {
    // FIXME(rustc/LLVM bug?) storing the ceiling in the resource de-optimizes
    // claims (the ceiling value gets loaded at runtime rather than inlined)
    // ceiling: u8,
    data: UnsafeCell<T>,
}

impl<T> Resource<T> {
    pub const fn new(value: T) -> Self {
        Resource { data: UnsafeCell::new(value) }
    }

    #[inline(always)]
    pub unsafe fn claim<R, F>(
        &'static self,
        ceiling: u8,
        nvic_prio_bits: u8,
        t: &mut Threshold,
        f: F,
    ) -> R
    where
        F: FnOnce(&Static<T>, &mut Threshold) -> R,
    {
        claim(&self.data, ceiling, nvic_prio_bits, t, f, |data| {
            Static::ref_(&*data.get())
        })
    }

    #[inline(always)]
    pub unsafe fn claim_mut<R, F>(
        &'static self,
        ceiling: u8,
        nvic_prio_bits: u8,
        t: &mut Threshold,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut Static<T>, &mut Threshold) -> R,
    {
        claim(&self.data, ceiling, nvic_prio_bits, t, f, |data| {
            Static::ref_mut(&mut *data.get())
        })
    }

    pub fn get(&self) -> *mut T {
        self.data.get()
    }
}

unsafe impl<T> Sync for Resource<T>
where
    T: Send,
{
}

pub struct Threshold(u8);

impl Threshold {
    pub unsafe fn new(value: u8) -> Self {
        Threshold(value)
    }
}

impl !Send for Threshold {}

/// Sets an interrupt as pending
pub fn set_pending<I>(interrupt: I)
where
    I: Nr,
{
    // NOTE(safe) atomic write
    let nvic = unsafe { &*cortex_m::peripheral::NVIC.get() };
    nvic.set_pending(interrupt);
}

#[macro_export]
macro_rules! task {
    ($NAME:ident, $body:path) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn $NAME() {
            let f: fn($crate::Threshold, ::$NAME::Resources) = $body;

            f(
                $crate::Threshold::new(::$NAME::$NAME),
                ::$NAME::Resources::new(),
            );
        }
    };
    ($NAME:ident, $body:path, $local:ident {
        $($var:ident: $ty:ty = $expr:expr;)+
    }) => {
        struct $local {
            $($var: $ty,)+
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn $NAME() {
            let f: fn(
                $crate::Threshold,
                &mut $local,
                ::$NAME::Resources,
            ) = $body;

            static mut LOCAL: $local = $local {
                $($var: $expr,)+
            };

            f(
                $crate::Threshold::new(::$NAME::$NAME),
                &mut LOCAL,
                ::$NAME::Resources::new(),
            );
        }
    };
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
pub enum Exception {
    /// System service call via SWI instruction
    SVCALL,
    /// Pendable request for system service
    PENDSV,
    /// System tick timer
    SYS_TICK,
}

impl Exception {
    #[doc(hidden)]
    pub fn nr(&self) -> usize {
        match *self {
            Exception::SVCALL => 11,
            Exception::PENDSV => 14,
            Exception::SYS_TICK => 15,
        }
    }
}
