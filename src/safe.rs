//! Safe implementations for Selfie and SelfieMut that do not rely on anything internal to it

use crate::refs::*;
use crate::{Selfie, SelfieMut};
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use stable_deref_trait::StableDeref;

impl<'a, P, R> Selfie<'a, P, R>
where
    P: StableDeref + 'a,
    R: for<'this> RefType<'this>,
    P::Target: 'a,
{
    /// Returns a copy of the reference type, if it implements [`Copy`].
    ///
    /// The actual reference type cannot be directly borrowed, as it's lifetime is
    /// self-referential. If you want to access it without making a copy, see the
    /// [`with_referential`](Selfie::with_referential) method.
    ///
    /// # Example
    ///
    /// This example stores both an owned `String` and a [`str`] slice pointing
    /// into it.
    ///
    /// ```
    /// use core::pin::Pin;
    /// use selfie::{refs::Ref, Selfie};
    ///
    /// let data: Pin<String> = Pin::new("Hello, world!".to_owned());
    /// let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |s| &s[0..5]);
    ///
    /// assert_eq!("Hello", selfie.referential());
    /// ```
    #[inline]
    pub fn referential<'s>(&'s self) -> <R as RefType<'s>>::Ref
    where
        <R as RefType<'s>>::Ref: Copy,
    {
        self.with_referential(|r| *r)
    }
}

impl<'a, P, R> Debug for Selfie<'a, P, R>
where
    P::Target: Debug,
    for<'this> <R as RefType<'this>>::Ref: Debug,
    P: 'a + StableDeref,
    R: for<'this> RefType<'this>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("Selfie")
                .field("owned", &self.owned())
                .field("referential", referential)
                .finish()
        })
    }
}

impl<'a, P, R> SelfieMut<'a, P, R>
where
    P: StableDeref + DerefMut + 'a,
    R: for<'this> RefType<'this>,
    P::Target: 'a,
{
    /// Returns a copy of the reference type, if it implements [`Copy`].
    ///
    /// The actual reference type cannot be directly borrowed, as it's lifetime is
    /// self-referential. If you want to access it without making a copy, see the
    /// [`with_referential`](SelfieMut::with_referential) method.
    ///
    /// # Example
    ///
    /// This example stores both an owned `String` and a [`str`] slice pointing
    /// into it.
    ///
    /// ```
    /// use core::pin::Pin;
    /// use selfie::{refs::Ref, SelfieMut};
    ///
    /// let data: Pin<String> = Pin::new("Hello, world!".to_owned());
    /// let selfie: SelfieMut<String, Ref<str>> = SelfieMut::new(data, |s| &Pin::into_inner(s)[0..5]);
    ///
    /// assert_eq!("Hello", selfie.referential());
    /// ```
    #[inline]
    pub fn referential<'s>(&'s self) -> <R as RefType<'s>>::Ref
    where
        <R as RefType<'s>>::Ref: Copy,
    {
        self.with_referential(|r| *r)
    }
}

impl<'a, P, R> Debug for SelfieMut<'a, P, R>
where
    for<'this> <R as RefType<'this>>::Ref: Debug,
    P: StableDeref + DerefMut + 'a,
    R: for<'this> RefType<'this>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("Selfie")
                .field("referential", referential)
                .finish()
        })
    }
}
