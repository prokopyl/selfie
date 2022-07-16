use core::fmt::{Debug, Display, Formatter};
use core::pin::Pin;

/// An error wrapper containing both an error and an owned value.
///
/// This is used by methods such as [`Selfie::try_new`](crate::Selfie::try_new) to allow recovering the owned pointer if
/// its reference handler failed.
pub struct SelfieError<P, E> {
    /// The owned value.
    pub owned: Pin<P>,
    /// The error value.
    pub error: E,
}

impl<P, E: Debug> Debug for SelfieError<P, E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}

impl<P, E: Display> Display for SelfieError<P, E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}

#[cfg(feature = "std")]
mod std_impl {
    extern crate std;
    use crate::SelfieError;
    use std::error::Error;

    impl<P, E: Error + 'static> Error for SelfieError<P, E> {
        #[inline]
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.error)
        }
    }
}
