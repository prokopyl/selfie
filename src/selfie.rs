use crate::refs::*;
use crate::unsafe_selfie::UnsafeSelfie;
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

pub struct Selfie<'a, P: 'a, R: for<'this> RefType<'this> + ?Sized> {
    inner: UnsafeSelfie<'a, P, R>,
}

impl<'a, P: StableDeref + 'a, R: for<'this> RefType<'this> + ?Sized> Selfie<'a, P, R> {
    pub fn new(
        owned: Pin<P>,
        handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        Self {
            inner: UnsafeSelfie::new_ref(owned, handler),
        }
    }

    #[inline]
    pub fn owned(&self) -> &P::Target {
        // SAFETY: referential has been created using a shared reference
        unsafe { self.inner.owned() }
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        self.inner.referential()
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.inner.into_inner()
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
    inner: UnsafeSelfie<'a, P, R>,
}

impl<'a, P: StableDeref + DerefMut + 'a, R: for<'this> RefType<'this> + ?Sized>
    SelfieMut<'a, P, R>
{
    #[inline]
    pub fn new(
        owned: Pin<P>,
        handler: for<'this> fn(Pin<&'this mut P::Target>) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        Self {
            inner: UnsafeSelfie::new_mut(owned, handler),
        }
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        self.inner.referential()
    }

    #[inline]
    pub fn referential_mut(&mut self) -> &mut <R as RefType<'a>>::Ref {
        // SAFETY: TODO?
        unsafe { self.inner.referential_mut() }
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.inner.into_inner()
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
