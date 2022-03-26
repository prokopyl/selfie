use crate::refs::*;
use crate::utils::*;
use core::fmt::{Debug, Formatter};
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

mod safe {}

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
        // SAFETY: Downcasting is safe here, becasue Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(&self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this mut <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Downcasting is safe here, becasue Ref is actually 's, not 'a
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
        // SAFETY: Downcasting is safe here, becasue Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(&self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this mut <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Downcasting is safe here, becasue Ref is actually 's, not 'a
        let referential = unsafe { downcast_mut::<'s, 'a, R>(&mut self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.pinned
    }
}

impl<'a, P: StableDeref + DerefMut + 'a, R: for<'this> RefType<'this> + ?Sized> Debug
    for SelfieMut<'a, P, R>
where
    for<'this> <R as RefType<'this>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("Selfie")
                .field("referential", referential)
                .finish()
        })
    }
}
