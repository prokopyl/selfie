use core::fmt::{Debug, Display, Formatter};
use core::pin::Pin;

/// An error wrapper containing both an error and an owned value.
///
/// This is used by methods such as [`crate::Selfie::try_new`] to allow recovering the owned pointer if
/// its reference handler failed.
pub struct SelfieError<O, E> {
    /// The owned value.
    pub owned: Pin<O>,
    /// The error value.
    pub error: E,
}

impl<O, E: Debug> Debug for SelfieError<O, E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}

impl<O, E: Display> Display for SelfieError<O, E> {
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

    impl<O, E: Error + 'static> Error for SelfieError<O, E> {
        #[inline]
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.error)
        }
    }
}
