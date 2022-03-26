//! This internal module contains the implementation details for Selfie and SelfieMut.
//!
//! **Do not make any change here without adding new regression and/or MIRI tests!**
//!
//! I do not trust myself in here, and neither should you.

#![allow(unsafe_code)] // I'll be glad to remove this the day self-referential structs can be implemented in Safe Rust

use crate::refs::*;
use crate::utils::*;
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

pub struct Selfie<'a, P: 'a, R: for<'this> RefType<'this> + ?Sized> {
    // SAFETY: enforce drop order!
    // SAFETY: Note that Ref's lifetime isn't actually ever 'a: it is the unnameable 'this instead.
    // Marking it as 'a is a trick to be able to store it and still name the whole type.
    // It is *absolutely* unsound to ever use this field as 'a, it should immediately be casted
    // to and from 'this instead.
    referential: <R as RefType<'a>>::Ref,
    owned: Pin<P>,
}

impl<'a: 'b, 'b: 'a, P: StableDeref + 'a, R: for<'this> RefType<'this> + ?Sized> Selfie<'a, P, R> {
    pub fn new(
        owned: Pin<P>,
        handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();

        Self {
            referential: handler(detached),
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
}

pub struct SelfieMut<'a, P: 'a, R: for<'this> RefType<'this> + ?Sized> {
    // SAFETY: enforce drop order!
    referential: <R as RefType<'a>>::Ref,
    pinned: Pin<P>,
}

impl<'a, P: StableDeref + DerefMut + 'a, R: for<'this> RefType<'this> + ?Sized>
    SelfieMut<'a, P, R>
{
    #[inline]
    pub fn new(
        mut pinned: Pin<P>,
        handler: for<'this> fn(Pin<&'this mut P::Target>) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime_mut(pinned.as_mut()) };

        Self {
            referential: handler(detached),
            pinned,
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
        self.pinned
    }
}
