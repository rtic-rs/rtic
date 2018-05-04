use core::cmp::Ordering;
use core::{mem, ptr};

use instant::Instant;

#[doc(hidden)]
pub struct Node<T>
where
    T: 'static,
{
    #[cfg(feature = "timer-queue")]
    pub baseline: Instant,
    pub payload: T,
}
