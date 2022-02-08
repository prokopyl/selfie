use crate::refs::*;
use crate::utils::*;
use core::fmt::{Debug, Formatter};
use core::ops::{Deref, DerefMut};
use core::pin::Pin;

pub struct Selfie<'a, P: 'a, R: for<'this> RefType<'this> + ?Sized> {
    // SAFETY: enforce drop order!
    referential: <R as RefType<'a>>::Ref,
    pinned: Pin<P>,
}

impl<'a, P: Deref + 'a, R: for<'this> RefType<'this> + ?Sized> Selfie<'a, P, R> {
    pub fn new(
        pinned: Pin<P>,
        handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        // SAFETY: derefd is pinned and cannot move, and this struct guarantees its lifetime
        let derefd = unsafe { detach_lifetime(pinned.as_ref()) }.get_ref();

        let referential = handler(derefd);

        Self {
            referential,
            pinned,
        }
    }

    #[inline]
    pub fn owned(&self) -> &P::Target {
        self.pinned.as_ref().get_ref()
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        &self.referential
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.pinned
    }
}

impl<'a, P: 'a + Deref, R: for<'this> RefType<'this> + ?Sized> Debug for Selfie<'a, P, R>
where
    P::Target: Debug,
    <R as RefType<'a>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Selfie")
            .field("owned", &self.owned())
            .field("reference", self.referential())
            .finish()
    }
}

pub struct SelfieMut<'a, P: 'a, R: for<'this> RefType<'this> + ?Sized> {
    // SAFETY: enforce drop order!
    referential: <R as RefType<'a>>::Ref,
    pinned: Pin<P>,
}

impl<'a, P: DerefMut + 'a, R: for<'this> RefType<'this> + ?Sized> SelfieMut<'a, P, R> {
    #[inline]
    pub fn new(
        mut pinned: Pin<P>,
        handler: for<'this> fn(Pin<&'this mut P::Target>) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        // SAFETY: derefd is pinned and cannot move, and this struct guarantees its lifetime
        let derefd = unsafe { detach_lifetime_mut(pinned.as_mut()) };

        let referential = handler(derefd);

        Self {
            referential,
            pinned,
        }
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        &self.referential
    }

    #[inline]
    pub fn referential_mut(&mut self) -> &mut <R as RefType<'a>>::Ref {
        &mut self.referential
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.pinned
    }
}

impl<'a, P: DerefMut + 'a, R: for<'this> RefType<'this> + ?Sized> Debug for SelfieMut<'a, P, R>
where
    <R as RefType<'a>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SelfieMut")
            .field("reference", self.referential())
            .finish()
    }
}
