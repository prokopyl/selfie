use crate::refs::*;
use crate::utils::*;
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

pub struct Selfie<'a, P: 'a, R: for<'this> RefType<'this> + ?Sized> {
    // SAFETY: enforce drop order!
    referential: <R as RefType<'a>>::Ref,
    owned: Pin<P>,
}

impl<'a, P: StableDeref + 'a, R: for<'this> RefType<'this> + ?Sized> Selfie<'a, P, R> {
    pub fn new(
        owned: Pin<P>,
        handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let derefd = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();

        let referential = handler(derefd);

        Self { referential, owned }
    }

    #[inline]
    pub fn owned(&self) -> &P::Target {
        self.owned.as_ref().get_ref()
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        &self.referential
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.owned
    }
}

impl<'a, P: 'a + StableDeref, R: for<'this> RefType<'this> + ?Sized> Debug for Selfie<'a, P, R>
where
    P::Target: Debug,
    <R as RefType<'a>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Selfie")
            .field("owned", &self.owned())
            .field("referential", self.referential())
            .finish()
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

impl<'a, P: StableDeref + DerefMut + 'a, R: for<'this> RefType<'this> + ?Sized> Debug
    for SelfieMut<'a, P, R>
where
    <R as RefType<'a>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SelfieMut")
            .field("referential", self.referential())
            .finish()
    }
}
