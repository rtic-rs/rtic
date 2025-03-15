use core::{future::poll_fn, task::Poll};

use super::Channel;

#[cfg(feature = "defmt-03")]
use crate::defmt;

/// Possible receive errors.
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReceiveError {
    /// Error state for when all senders has been dropped.
    NoSender,
    /// Error state for when the queue is empty.
    Empty,
}

/// A receiver of the channel. There can only be one receiver at any time.
pub struct Receiver<'a, T, const N: usize>(pub(crate) &'a Channel<T, N>);

unsafe impl<T, const N: usize> Send for Receiver<'_, T, N> {}

impl<T, const N: usize> core::fmt::Debug for Receiver<'_, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Receiver")
    }
}

#[cfg(feature = "defmt-03")]
impl<T, const N: usize> defmt::Format for Receiver<'_, T, N> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Receiver",)
    }
}

impl<T, const N: usize> Receiver<'_, T, N> {
    /// Receives a value if there is one in the channel, non-blocking.
    pub fn try_recv(&mut self) -> Result<T, ReceiveError> {
        // Try to get a ready slot.
        let ready_slot = self.0.receive_value();

        if let Some(value) = ready_slot {
            Ok(value)
        } else if self.is_closed() {
            Err(ReceiveError::NoSender)
        } else {
            Err(ReceiveError::Empty)
        }
    }

    /// Receives a value, waiting if the queue is empty.
    /// If all senders are dropped this will error with `NoSender`.
    pub async fn recv(&mut self) -> Result<T, ReceiveError> {
        // There was nothing in the queue, setup the waiting.
        poll_fn(|cx| {
            // Register waker.
            // TODO: Should it happen here or after the if? This might cause a spurious wake.
            self.0.register_receiver_waker(cx.waker());

            // Try to dequeue.
            match self.try_recv() {
                Ok(val) => {
                    return Poll::Ready(Ok(val));
                }
                Err(ReceiveError::NoSender) => {
                    return Poll::Ready(Err(ReceiveError::NoSender));
                }
                _ => {}
            }

            Poll::Pending
        })
        .await
    }

    /// Returns true if there are no `Sender`s.
    pub fn is_closed(&self) -> bool {
        self.0.num_senders() == 0
    }

    /// Is the queue full.
    pub fn is_full(&self) -> bool {
        // SAFETY: `self.0.readyq` is not called recursively.
        unsafe { self.0.readyq(|q| q.is_full()) }
    }

    /// Is the queue empty.
    pub fn is_empty(&self) -> bool {
        // SAFETY: `self.0.readyq` is not called recursively.
        unsafe { self.0.readyq(|q| q.is_empty()) }
    }
}

impl<T, const N: usize> Drop for Receiver<'_, T, N> {
    fn drop(&mut self) {
        self.0.drop_receiver();
    }
}
