use crate::refs::*;
use crate::unsafe_selfie::UnsafePinnedSelfie;
use crate::StableOwned;
use core::fmt::{Debug, Formatter};
use core::pin::Pin;

pub struct PinnedSelfie<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> {
    inner: UnsafePinnedSelfie<'a, T, R>,
}

impl<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> PinnedSelfie<'a, T, R> {
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub fn new<P: 'a + StableOwned<Self>>(
        owned: T,
        handler: for<'this> fn(&'this T) -> <R as RefType<'this>>::Ref,
    ) -> Pin<P> {
        let this = Self {
            // SAFETY: we are initializing referential before the value is exposed
            inner: unsafe { UnsafePinnedSelfie::new_uninitialized(owned) },
        };

        let mut pinned = P::new_pinned(this);

        unsafe {
            P::pin_as_mut(&mut pinned)
                .inner()
                .write_referential(handler)
        };

        pinned
    }

    #[inline]
    fn inner(self: Pin<&mut Self>) -> Pin<&mut UnsafePinnedSelfie<'a, T, R>> {
        // SAFETY: accessing inner does not move anything
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }

    #[inline]
    pub fn owned(&self) -> &T {
        // SAFETY: referential has been created using a shared reference
        unsafe { self.inner.owned() }
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        self.inner.referential()
    }

    #[inline]
    pub fn into_inner<P: StableOwned<Self>>(mut this: Pin<P>) -> T {
        // SAFETY: we ensure the referential is never accessed after this
        unsafe { P::pin_as_mut(&mut this).inner().drop_referential() };

        // SAFETY: we dropped the referential, T is Unpin again
        let this = unsafe { Pin::into_inner_unchecked(this) };
        P::unwrap(this).inner.into_inner()
    }
}

impl<'a, T: 'a + Debug + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> Debug
    for PinnedSelfie<'a, T, R>
where
    <R as RefType<'a>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PinnedSelfie")
            .field("owned", &self.owned())
            .field("reference", self.referential())
            .finish()
    }
}

pub struct PinnedSelfieMut<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> {
    inner: UnsafePinnedSelfie<'a, T, R>,
}

impl<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> PinnedSelfieMut<'a, T, R> {
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub fn new<P: 'a + StableOwned<Self>>(
        owned: T,
        handler: for<'this> fn(&'this mut T) -> <R as RefType<'this>>::Ref,
    ) -> Pin<P> {
        let this = Self {
            // SAFETY: we are initializing referential before the value is exposed
            inner: unsafe { UnsafePinnedSelfie::new_uninitialized(owned) },
        };

        let mut pinned = P::new_pinned(this);

        unsafe {
            P::pin_as_mut(&mut pinned)
                .inner()
                .write_referential_mut(handler)
        };

        pinned
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        self.inner.referential()
    }

    #[inline]
    pub fn referential_mut(self: Pin<&mut Self>) -> &mut <R as RefType<'a>>::Ref {
        self.inner().referential_mut()
    }

    #[inline]
    fn inner(self: Pin<&mut Self>) -> Pin<&mut UnsafePinnedSelfie<'a, T, R>> {
        // SAFETY: accessing inner does not move anything
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }

    #[inline]
    pub fn into_inner<P: StableOwned<Self>>(mut this: Pin<P>) -> T {
        // SAFETY: we ensure the referential is never accessed after this
        unsafe { P::pin_as_mut(&mut this).inner().drop_referential() };

        // SAFETY: we dropped the referential, T is Unpin again
        let this = unsafe { Pin::into_inner_unchecked(this) };
        P::unwrap(this).inner.into_inner()
    }
}

impl<'a, T: 'a + Debug + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> Debug
    for PinnedSelfieMut<'a, T, R>
where
    <R as RefType<'a>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PinnedSelfieMut")
            .field("reference", self.referential())
            .finish()
    }
}
