#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
extern crate static_ref;

use core::cell::UnsafeCell;

pub use cortex_m::interrupt::free as _free;
pub use cortex_m::asm::{bkpt, wfi};
pub use static_ref::Static;
#[cfg(not(thumbv6m))]
use cortex_m::register::{basepri_max, basepri};

pub const MAX_PRIORITY: u8 = 1 << PRIORITY_BITS;
const PRIORITY_BITS: u8 = 4;

#[cfg(not(thumbv6m))]
macro_rules! barrier {
    () => { asm!("" ::: "memory" : "volatile") }
}

unsafe fn claim<T, U, R, F, G>(
    value: T,
    g: G,
    f: F,
    ceiling: u8,
    threshold: &Threshold,
) -> R
where
    G: FnOnce(T) -> U,
    F: FnOnce(U, &Threshold) -> R,
{
    if ceiling > threshold.value {
        match () {
            #[cfg(thumbv6m)]
            () => _free(|_| f(g(value), &Threshold::max())),
            #[cfg(not(thumbv6m))]
            () => {
                if ceiling == MAX_PRIORITY {
                    _free(|_| f(g(value), &Threshold::max()))
                } else {
                    let old = basepri::read();
                    basepri_max::write(_logical2hw(ceiling));
                    barrier!();
                    let ret = f(g(value), &Threshold { value: ceiling });
                    barrier!();
                    basepri::write(old);
                    ret
                }
            }
        }
    } else {
        f(g(value), threshold)
    }
}

pub struct Peripheral<P>
where
    P: 'static,
{
    ceiling: u8,
    peripheral: cortex_m::peripheral::Peripheral<P>,
}

unsafe impl<P> Sync for Peripheral<P> {}

impl<P> Peripheral<P> {
    pub const unsafe fn new(
        p: cortex_m::peripheral::Peripheral<P>,
        ceiling: u8,
    ) -> Self {
        Peripheral {
            peripheral: p,
            ceiling: ceiling,
        }
    }

    pub unsafe fn claim<R, F>(&'static self, threshold: &Threshold, f: F) -> R
    where
        F: FnOnce(&P, &Threshold) -> R,
    {
        claim(
            &self.peripheral,
            |data| &*data.get(),
            f,
            self.ceiling,
            threshold,
        )
    }
}

// FIXME ceiling should be a field of this struct but today that causes
// misoptimizations. The misoptimizations seem to be due to a rustc / LLVM bug
pub struct Resource<T> {
    data: UnsafeCell<T>,
}

impl<T> Resource<T> {
    pub const fn new(value: T) -> Self {
        Resource { data: UnsafeCell::new(value) }
    }

    /// # Unsafety
    ///
    /// - Caller must ensure that this method is called from a task with
    ///   priority `P` where `Resource.ceiling <= P`.
    pub unsafe fn claim<R, F>(
        &'static self,
        ceiling: u8,
        threshold: &Threshold,
        f: F,
    ) -> R
    where
        F: FnOnce(&Static<T>, &Threshold) -> R,
    {
        claim(
            &self.data,
            |data| Static::ref_(&*data.get()),
            f,
            ceiling,
            threshold,
        )
    }

    /// # Unsafety
    ///
    /// - Caller must ensure that this method is called from a task with
    ///   priority `P` where `Resource.ceiling <= P`.
    ///
    /// - Caller must take care to not break Rust borrowing rules
    pub unsafe fn claim_mut<R, F>(
        &'static self,
        ceiling: u8,
        threshold: &Threshold,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut Static<T>, &Threshold) -> R,
    {
        claim(
            &self.data,
            |data| Static::ref_mut(&mut *data.get()),
            f,
            ceiling,
            threshold,
        )
    }

    pub unsafe fn get(&'static self) -> *mut T {
        self.data.get()
    }
}

unsafe impl<T> Sync for Resource<T>
where
    T: Send,
{
}

pub struct Threshold {
    value: u8,
}

impl Threshold {
    pub unsafe fn max() -> Self {
        Threshold { value: 1 << PRIORITY_BITS }
    }

    pub unsafe fn new(value: u8) -> Self {
        Threshold { value: value }
    }
}

/// Fault and system exceptions
#[allow(non_camel_case_types)]
#[doc(hidden)]
pub enum Exception {
    /// Memory management.
    MEN_MANAGE,
    /// Pre-fetch fault, memory access fault.
    BUS_FAULT,
    /// Undefined instruction or illegal state.
    USAGE_FAULT,
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
            Exception::MEN_MANAGE => 4,
            Exception::BUS_FAULT => 5,
            Exception::USAGE_FAULT => 6,
            Exception::SVCALL => 11,
            Exception::PENDSV => 14,
            Exception::SYS_TICK => 15,
        }
    }
}

pub fn atomic<R, F>(f: F) -> R
where
    F: FnOnce(&Threshold) -> R,
{
    _free(|_| unsafe { f(&Threshold::max()) })
}

pub fn _logical2hw(i: u8) -> u8 {
    ((1 << PRIORITY_BITS) - i) << (8 - PRIORITY_BITS)
}

#[macro_export]
macro_rules! rtfm {
    (device: $device:ident,

     peripherals: {
         $($pvar:ident @ $pceiling:expr,)*
     },

     // FIXME ceiling should not be required; instead it should be derived from
     // the other information. This is doable but causes misoptimizations. A fix
     // requires better const support.
     resources: {
         $($rvar:ident: $rty:ty = $rval:expr; @ $rceiling:expr,)*
     },

     init: {
         body: $init:path,
         peripherals: [$($inperipheral:ident,)*],
     },

     idle: {
         body: $idle:path,
         local: {
             $($lvar:ident: $lty:ty = $lval:expr;)*
         },
         peripherals: [$($idperipheral:ident,)*],
         resources: [$($idresource:ident,)*],
     },

     exceptions: {
         $(
             $EXCEPTION:ident: {
                 body: $ebody:path,
                 priority: $epriority:expr,
                 local: {
                     $($elvar:ident: $elty:ty = $elval:expr;)*
                 },
                 peripherals: [$($eperipheral:ident,)*],
                 resources: [$($eresource:ident,)*],
             },
         )*
     },

     interrupts: {
         $(
             $INTERRUPT:ident: {
                 body: $ibody:path,
                 priority: $ipriority:expr,
                 enabled: $enabled:expr,
                 local: {
                     $($ilvar:ident: $ilty:ty = $ilval:expr;)*
                 },
                 peripherals: [$($iperipheral:ident,)*],
                 resources: [$($iresource:ident,)*],
             },
         )*
     },
    ) => {
        mod _ceiling {
            #![allow(non_upper_case_globals)]
            #![allow(dead_code)]

            $(
                pub const $rvar: u8 = $rceiling;
            )*

            $(
                pub const $pvar: u8 = $pceiling;
            )*

            mod verify {
                #![allow(dead_code)]
                #![deny(const_err)]

                $(
                    const $rvar: u8 = $crate::MAX_PRIORITY - $rceiling;
                )*

                $(
                    const $pvar: u8 = $crate::MAX_PRIORITY - $pceiling;
                )*
            }
        }

        pub mod _resource {
            $(
                #[allow(non_upper_case_globals)]
                pub static $rvar: $crate::Resource<$rty> =
                    $crate::Resource::new($rval);
            )*

            $(
                #[allow(non_camel_case_types)]
                pub struct $rvar { _0: () }

                impl $rvar {
                    pub unsafe fn new() -> Self {
                        $rvar { _0: () }
                    }

                    pub fn claim<R, F>(
                        &self,
                        t: &$crate::Threshold,
                        f: F,
                    ) -> R
                    where
                        F: FnOnce(
                            &$crate::Static<$rty>,
                            &$crate::Threshold,
                        ) -> R,
                    {
                        unsafe {
                            $rvar.claim(super::_ceiling::$rvar, t, f)
                        }
                    }

                    pub fn claim_mut<R, F>(
                        &mut self,
                        t: &$crate::Threshold,
                        f: F,
                    ) -> R
                    where
                        F: FnOnce(
                            &mut $crate::Static<$rty>,
                            &$crate::Threshold,
                        ) -> R,
                    {
                        unsafe {
                            $rvar.claim_mut(super::_ceiling::$rvar, t, f)
                        }
                    }
                }
            )*
        }

        mod _peripheral {
            // Peripherals
            $(
                static $pvar:
                    $crate::Peripheral<::$device::$pvar> = unsafe {
                        $crate::Peripheral::new(
                            ::$device::$pvar,
                            $pceiling,
                        )
                    };

                pub struct $pvar { _0: () }

                impl $pvar {
                    pub unsafe fn new() -> Self {
                        $pvar { _0: () }
                    }

                    pub fn claim<R, F>(
                        &self,
                        t: &$crate::Threshold,
                        f: F,
                    ) -> R
                    where
                        F: FnOnce(
                            &::$device::$pvar,
                            &$crate::Threshold,
                        ) -> R,
                    {
                        unsafe {
                            $pvar.claim(t, f)
                        }
                    }
                }
            )*

        }

        $(
            #[allow(non_snake_case)]
            mod $EXCEPTION {
                pub struct Local {
                    $(pub $elvar: $crate::Static<$elty>,)*
                }

                pub struct Resources {
                    $(pub $eresource: super::_resource::$eresource,)*
                }

                pub struct Peripherals {
                    $(pub $eperipheral: super::_peripheral::$eperipheral,)*
                }
            }
        )*

        $(
            #[allow(non_snake_case)]
            mod $INTERRUPT {
                pub struct Local {
                    $(pub $ilvar: $crate::Static<$ilty>,)*
                }

                pub struct Resources {
                    $(pub $iresource: super::_resource::$iresource,)*
                }

                pub struct Peripherals {
                    $(pub $iperipheral: super::_peripheral::$iperipheral,)*
                }
            }
        )*

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            pub unsafe extern "C" fn $EXCEPTION() {
                // verify task priority
                #[allow(dead_code)]
                #[deny(const_err)]
                const $EXCEPTION: (u8, u8) = (
                    $epriority - 1,
                    $crate::MAX_PRIORITY - $epriority,
                );

                // verify ceiling correctness
                $(
                    #[allow(dead_code)]
                    #[allow(non_upper_case_globals)]
                    #[deny(const_err)]
                    const $eresource: u8 = _ceiling::$eresource - $epriority;
                )*

                $(
                    #[allow(dead_code)]
                    #[allow(non_upper_case_globals)]
                    #[deny(const_err)]
                    const $eperipheral: u8 =
                        _ceiling::$eperipheral - $epriority;
                )*

                // check that the interrupt handler exists
                let _ = $crate::Exception::$EXCEPTION;

                // type check
                // TODO Local should not appear in the signature if no local
                // data is declared
                // TODO Resources should on appear in the signature if no
                // resources has been declared
                let f: fn(&$crate::Threshold,
                          &mut $EXCEPTION::Local,
                          &$EXCEPTION::Peripherals,
                          &mut $EXCEPTION::Resources) = $ebody;

                static mut LOCAL: $EXCEPTION::Local = unsafe {
                    $EXCEPTION::Local {
                        $($elvar: $crate::Static::new($elval),)*
                    }
                };

                f(
                    &$crate::Threshold::new($epriority),
                    &mut LOCAL,
                    &$EXCEPTION::Peripherals {
                        $($eperipheral: _peripheral::$eperipheral::new(),)*
                    },
                    &mut $EXCEPTION::Resources {
                        $($eresource: _resource::$eresource::new(),)*
                    },
                )
            }
        )*

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            pub unsafe extern "C" fn $INTERRUPT() {
                // verify task priority
                #[allow(dead_code)]
                #[deny(const_err)]
                const $INTERRUPT: (u8, u8) = (
                    $ipriority - 1,
                    $crate::MAX_PRIORITY - $ipriority,
                );

                // verify ceiling correctness
                $(
                    #[allow(dead_code)]
                    #[allow(non_upper_case_globals)]
                    #[deny(const_err)]
                    const $iresource: u8 = _ceiling::$iresource - $ipriority;
                )*

                // check that the interrupt handler exists
                let _ = $device::interrupt::Interrupt::$INTERRUPT;

                // type check
                // TODO Local should not appear in the signature if no local
                // data is declared
                // TODO Resources should on appear in the signature if no
                // resources has been declared
                let f: fn(&$crate::Threshold,
                          &mut $INTERRUPT::Local,
                          &$INTERRUPT::Peripherals,
                          &mut $INTERRUPT::Resources) = $ibody;

                #[allow(unused_unsafe)]
                static mut LOCAL: $INTERRUPT::Local = unsafe {
                    $INTERRUPT::Local {
                        $($ilvar: $crate::Static::new($ilval)),*
                    }
                };

                f(
                    &$crate::Threshold::new($ipriority),
                    &mut LOCAL,
                    &$INTERRUPT::Peripherals {
                        $($iperipheral: _peripheral::$iperipheral::new(),)*
                    },
                    &mut $INTERRUPT::Resources {
                        $($iresource: _resource::$iresource::new(),)*
                    },
                )
            }
        )*

        #[allow(non_snake_case)]
        mod init {
            pub struct Peripherals<'a> {
                pub _0: ::core::marker::PhantomData<&'a ()>,
                $(pub $inperipheral: &'a ::$device::$inperipheral,)*
            }

            pub struct Resources<'a> {
                pub _0: ::core::marker::PhantomData<&'a ()>,
                $(pub $rvar: &'a mut $crate::Static<$rty>,)*
            }
        }

        #[allow(non_snake_case)]
        mod idle {
            pub struct Local {
                $(pub $lvar: $crate::Static<$lty>,)*
            }

            pub struct Peripherals {
                $(pub $idperipheral: super::_peripheral::$idperipheral,)*
            }

            pub struct Resources {
                $(pub $idresource: super::_resource::$idresource,)*
            }
        }

        fn main() {
            // type check
            let init: fn(
                init::Peripherals,
                init::Resources,
            ) = $init;
            // TODO Local should not appear in the signature if no local data is
            // declared
            // TODO Resources should on appear in the signature if no resources
            // has been declared
            let idle: fn(&$crate::Threshold,
                         &'static mut idle::Local,
                         &idle::Peripherals,
                         &mut idle::Resources) -> ! = $idle;

            $crate::_free(|cs| unsafe {
                init(
                    init::Peripherals {
                        _0: ::core::marker::PhantomData,
                        $(
                            $inperipheral: $device::$inperipheral.borrow(cs),
                        )*
                    },
                    init::Resources {
                        _0: ::core::marker::PhantomData,
                        $(
                            $rvar: $crate::Static::ref_mut(
                                &mut *_resource::$rvar.get(),
                            ),
                        )*
                    },
                );

                let _scb = $device::SCB.borrow(cs);

                $(
                    _scb.shpr[$crate::Exception::$EXCEPTION.nr() - 4].write(
                        $crate::_logical2hw($epriority),
                    );
                )*

                let _nvic = $device::NVIC.borrow(cs);

                $(
                    _nvic.set_priority(
                        $device::interrupt::Interrupt::$INTERRUPT,
                        $crate::_logical2hw($ipriority),
                    );
                )*

                $(
                    if $enabled {
                        _nvic.enable($device::interrupt::Interrupt::$INTERRUPT);
                    }
                )*
            });

            #[allow(unused_unsafe)]
            static mut LOCAL: idle::Local = unsafe {
                idle::Local {
                    $($lvar: $crate::Static::new($lval)),*
                }
            };

            unsafe {
                idle(
                    &$crate::Threshold::new(0),
                    &mut LOCAL,
                    &idle::Peripherals {
                        $(
                            $idperipheral: _peripheral::$idperipheral::new(),
                        )*
                    },
                    &mut idle::Resources {
                        $(
                            $idresource: _resource::$idresource::new(),
                        )*
                    },
                )
            }
        }
    }
}
