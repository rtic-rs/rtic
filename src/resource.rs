use core::marker::PhantomData;

#[cfg(not(armv6m))]
use cortex_m::register::basepri;

use typenum::type_operators::IsGreaterOrEqual;
use typenum::{Max, Maximum, True, Unsigned};

/// Token that represents the current priority level of a task
pub struct Priority<N> {
    _not_send_or_sync: PhantomData<*const ()>,
    _n: PhantomData<N>,
}

impl<N> Priority<N> {
    #[doc(hidden)]
    pub unsafe fn _new() -> Self {
        Priority {
            _not_send_or_sync: PhantomData,
            _n: PhantomData,
        }
    }
}

/// A resource shared between two or more tasks
pub unsafe trait Resource {
    #[doc(hidden)]
    const NVIC_PRIO_BITS: u8;

    /// The priority ceiling of the resource
    type Ceiling;

    /// The data protected by the resource
    type Data: 'static + Send;

    // The `static mut` variable that the resource protects fs
    #[doc(hidden)]
    unsafe fn _var() -> &'static mut Self::Data;

    /// Borrows the resource data for the span of the current priority
    #[inline(always)]
    fn borrow<'cs, P>(&'cs self, _p: &'cs Priority<P>) -> &'cs Self::Data
    where
        P: IsGreaterOrEqual<Self::Ceiling, Output = True>,
    {
        unsafe { Self::_var() }
    }

    /// Mutably borrows the resource data for the span of the current priority
    #[inline(always)]
    fn borrow_mut<'cs, P>(&'cs mut self, _p: &'cs Priority<P>) -> &'cs mut Self::Data
    where
        P: IsGreaterOrEqual<Self::Ceiling, Output = True>,
    {
        unsafe { Self::_var() }
    }

    /// Creates a critical section, by raising the task priority, to access the resource data
    #[inline(always)]
    fn claim<'cs, R, F, P>(&self, _p: &mut Priority<P>, f: F) -> R
    where
        F: FnOnce(&Self::Data, &mut Priority<Maximum<P, Self::Ceiling>>) -> R,
        P: Max<Self::Ceiling> + Unsigned,
        Self::Ceiling: Unsigned,
    {
        unsafe {
            if P::to_u8() >= Self::Ceiling::to_u8() {
                f(Self::_var(), &mut Priority::_new())
            } else {
                let max = 1 << Self::NVIC_PRIO_BITS;
                let new = (max - Self::Ceiling::to_u8()) << (8 - Self::NVIC_PRIO_BITS);

                let old = basepri::read();
                basepri::write(new);
                let r = f(Self::_var(), &mut Priority::_new());
                basepri::write(old);
                r
            }
        }
    }

    /// Creates a critical section, by raising the task priority, to mutably access the resource
    /// data
    #[inline(always)]
    fn claim_mut<'cs, R, F, P>(&mut self, _p: &mut Priority<P>, f: F) -> R
    where
        F: FnOnce(&mut Self::Data, &mut Priority<Maximum<P, Self::Ceiling>>) -> R,
        P: Max<Self::Ceiling> + Unsigned,
        Self::Ceiling: Unsigned,
    {
        unsafe {
            if P::to_u8() >= Self::Ceiling::to_u8() {
                f(Self::_var(), &mut Priority::_new())
            } else {
                let max = 1 << Self::NVIC_PRIO_BITS;
                let new = (max - Self::Ceiling::to_u8()) << (8 - Self::NVIC_PRIO_BITS);

                let old = basepri::read();
                basepri::write(new);
                let r = f(Self::_var(), &mut Priority::_new());
                basepri::write(old);
                r
            }
        }
    }
}
