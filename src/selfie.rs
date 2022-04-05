//! This internal module contains the implementation details for Selfie and SelfieMut.
//!
//! **Do not make any change here without adding new regression, compile-fail and/or MIRI tests!**
//!
//! I do not trust myself in here, and neither should you.

#![allow(unsafe_code)] // I'll be glad to remove this the day self-referential structs can be implemented in Safe Rust

use crate::convert::{IntoReferential, IntoReferentialMut};
use crate::refs::*;
use crate::utils::*;
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

/// A self-referential struct with a shared reference (`R`) to an object owned by a pinned pointer (`P`).
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
/// assert_eq!("Hello, world!", selfie.owned());
/// ```
pub struct Selfie<'a, P, R>
where
    P: 'a,
    R: for<'this> RefType<'this>,
{
    // SAFETY: enforce drop order!
    // SAFETY: Note that Ref's lifetime isn't actually ever 'a: it is the unnameable 'this instead.
    // Marking it as 'a is a trick to be able to store it and still name the whole type.
    // It is *absolutely* unsound to ever use this field as 'a, it should immediately be casted
    // to and from 'this instead.
    referential: <R as RefType<'a>>::Ref,
    owned: Pin<P>,
}

impl<'a, P, R> Selfie<'a, P, R>
where
    P: StableDeref + 'a,
    R: for<'this> RefType<'this>,
    P::Target: 'a,
{
    pub fn new_with(owned: Pin<P>, handler: impl IntoReferential<P, R>) -> Self {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();

        Self {
            referential: handler.into_referential(detached),
            owned,
        }
    }

    #[inline]
    pub fn owned(&self) -> &P::Target {
        self.owned.as_ref().get_ref()
    }

    #[inline]
    pub fn with_referential<'s, F, T>(&'s self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(&self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this mut <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_mut::<'s, 'a, R>(&mut self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.owned
    }

    #[inline]
    pub fn map<R2: for<'this> RefType<'this>>(
        self,
        mapper: for<'this> fn(
            <R as RefType<'this>>::Ref,
            &'this P::Target,
        ) -> <R2 as RefType<'this>>::Ref,
    ) -> Selfie<'a, P, R2> {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();
        let referential = mapper(referential, detached);

        Selfie { owned, referential }
    }
}

pub struct SelfieMut<'a, P, R>
where
    P: 'a,
    R: for<'this> RefType<'this>,
{
    // SAFETY: enforce drop order!
    referential: <R as RefType<'a>>::Ref,
    owned: Pin<P>,
}

impl<'a, P, R> SelfieMut<'a, P, R>
where
    P: StableDeref + DerefMut + 'a,
    R: for<'this> RefType<'this>,
{
    pub fn new_with(mut owned: Pin<P>, handler: impl IntoReferentialMut<P, R>) -> Self {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime_mut(owned.as_mut()) };

        Self {
            referential: handler.into_referential(detached),
            owned,
        }
    }

    #[inline]
    pub fn with_referential<'s, F, T>(&'s self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(&self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this mut <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_mut::<'s, 'a, R>(&mut self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.owned
    }

    #[inline]
    pub fn map<R2: for<'this> RefType<'this>>(
        self,
        mapper: impl for<'this> FnOnce(
            <R as RefType<'this>>::Ref,
            &'this (), // This is needed to constrain the lifetime TODO: find a way to remove this
        ) -> <R2 as RefType<'this>>::Ref,
    ) -> Selfie<'a, P, R2> {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        let referential = mapper(referential, &());

        Selfie { owned, referential }
    }
}
