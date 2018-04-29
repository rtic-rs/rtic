use core::marker::PhantomData;

#[cfg(not(armv6m))]
use cortex_m::register::basepri;

use typenum::{Max, Maximum, Unsigned};

pub struct Threshold<N>
where
    N: Unsigned,
{
    _not_send_or_sync: PhantomData<*const ()>,
    _n: PhantomData<N>,
}

impl<N> Threshold<N>
where
    N: Unsigned,
{
    pub unsafe fn new() -> Self {
        Threshold {
            _not_send_or_sync: PhantomData,
            _n: PhantomData,
        }
    }
}

pub unsafe trait Resource {
    #[doc(hidden)]
    const NVIC_PRIO_BITS: u8;
    type Ceiling: Unsigned;
    type Data: 'static + Send;

    #[doc(hidden)]
    unsafe fn get() -> &'static mut Self::Data;

    fn borrow<'cs>(&'cs self, _t: &'cs Threshold<Self::Ceiling>) -> &'cs Self::Data {
        unsafe { Self::get() }
    }

    fn borrow_mut<'cs>(&'cs mut self, _t: &'cs Threshold<Self::Ceiling>) -> &'cs mut Self::Data {
        unsafe { Self::get() }
    }

    fn claim<'cs, R, F, P>(&self, _t: &mut Threshold<P>, f: F) -> R
    where
        F: FnOnce(&Self::Data, &mut Threshold<Maximum<P, Self::Ceiling>>) -> R,
        P: Max<Self::Ceiling> + Unsigned,
        Maximum<P, Self::Ceiling>: Unsigned,
    {
        unsafe {
            if P::to_u8() >= Self::Ceiling::to_u8() {
                f(Self::get(), &mut Threshold::new())
            } else {
                let max = 1 << Self::NVIC_PRIO_BITS;
                let new = (max - Self::Ceiling::to_u8()) << (8 - Self::NVIC_PRIO_BITS);

                let old = basepri::read();
                basepri::write(new);
                let r = f(Self::get(), &mut Threshold::new());
                basepri::write(old);
                r
            }
        }
    }

    fn claim_mut<'cs, R, F, P>(&mut self, _t: &mut Threshold<P>, f: F) -> R
    where
        F: FnOnce(&mut Self::Data, &mut Threshold<Maximum<P, Self::Ceiling>>) -> R,
        P: Max<Self::Ceiling> + Unsigned,
        Maximum<P, Self::Ceiling>: Unsigned,
    {
        unsafe {
            if P::to_u8() >= Self::Ceiling::to_u8() {
                f(Self::get(), &mut Threshold::new())
            } else {
                let max = 1 << Self::NVIC_PRIO_BITS;
                let new = (max - Self::Ceiling::to_u8()) << (8 - Self::NVIC_PRIO_BITS);

                let old = basepri::read();
                basepri::write(new);
                let r = f(Self::get(), &mut Threshold::new());
                basepri::write(old);
                r
            }
        }
    }
}
