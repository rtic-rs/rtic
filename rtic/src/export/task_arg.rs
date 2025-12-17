/// A trait for types that can be passed as arguments when spawning tasks
///
/// This trait will only ever be implemented where type `Self::T` is `Self`
///
/// The global `my_task::spawn` requires its args to be `Send`. This trait has to
/// be used because we can not have a function with a where clause which
/// requires a concrete type to be `Send` if that type is not `Send`. The compiler
/// will error out on us. However hiding that behind a dummy trait which is
/// only implemented for that same type enables us to defer the error to when
/// the user erroneously tries to call the function.
pub trait TaskArg {
    /// This should always be same as `Self`
    type T;
    fn to(self) -> Self::T;
}

impl<T> TaskArg for T {
    type T = T;
    fn to(self) -> T {
        self
    }
}
