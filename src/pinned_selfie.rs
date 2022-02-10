use crate::refs::*;
use crate::utils::*;
use crate::StableOwned;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomPinned;
use core::pin::Pin;

pub struct PinnedSelfie<'a, T: 'a, R: for<'this> RefType<'this> + ?Sized> {
    referential: Option<<R as RefType<'a>>::Ref>,
    owned: T,
    _pinned: PhantomPinned,
}

impl<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> PinnedSelfie<'a, T, R> {
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub fn new<P: 'a + StableOwned<Self>>(
        owned: T,
        handler: for<'this> fn(&'this T) -> <R as RefType<'this>>::Ref,
    ) -> Pin<P> {
        let this = Self {
            owned,
            referential: None,
            _pinned: PhantomPinned,
        };

        let mut pinned = P::new_pinned(this);

        // SAFETY: we are not moving owned, only initializing referential
        let pinned_mut: &mut Self = unsafe { Pin::get_unchecked_mut(P::pin_as_mut(&mut pinned)) };
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(unsafe { detach_lifetime_ref(&pinned_mut.owned) });

        pinned_mut.referential = Some(referential);

        pinned
    }

    #[inline]
    pub fn owned(&self) -> &T {
        &self.owned
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        self.referential.as_ref().unwrap()
    }

    #[inline]
    pub fn referential_mut(self: Pin<&mut Self>) -> &mut <R as RefType<'a>>::Ref {
        // SAFETY: the referential type is not structural for this
        unsafe { self.get_unchecked_mut().referential.as_mut().unwrap() }
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

pub struct PinnedSelfieMut<'a, T: 'a, R: for<'this> RefType<'this> + ?Sized> {
    referential: Option<<R as RefType<'a>>::Ref>,
    owned: T,
    _pinned: PhantomPinned,
}

impl<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> PinnedSelfieMut<'a, T, R> {
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub fn new<P: 'a + StableOwned<Self>>(
        owned: T,
        handler: for<'this> fn(&'this mut T) -> <R as RefType<'this>>::Ref,
    ) -> Pin<P> {
        let this = Self {
            owned,
            referential: None,
            _pinned: PhantomPinned,
        };

        let mut pinned = P::new_pinned(this);

        // SAFETY: we are not moving owned, only initializing referential
        let pinned_mut = unsafe { Pin::get_unchecked_mut(P::pin_as_mut(&mut pinned)) };
        let (pinned_owned, pinned_referential) =
            (&mut pinned_mut.owned, &mut pinned_mut.referential);

        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(unsafe { detach_lifetime_ref_mut(pinned_owned) });
        *pinned_referential = Some(referential);

        pinned
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        self.referential.as_ref().unwrap()
    }

    #[inline]
    pub fn referential_mut(self: Pin<&mut Self>) -> &mut <R as RefType<'a>>::Ref {
        // SAFETY: the referential type is not structural for this
        unsafe { self.get_unchecked_mut().referential.as_mut().unwrap() }
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
