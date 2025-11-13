// TODO: What should we name this?

/// Dummy trait which will only ever be implemented where type T is Self
pub trait Dummy {
    /// This should always be same as `Self`
    type T;
    fn to(self) -> Self::T;
}

impl<T> Dummy for T {
    type T = T;
    fn to(self) -> T {
        self
    }
}
