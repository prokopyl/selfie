use crate::refs::*;
use crate::utils::{detach_lifetime_mut, detach_lifetime_ref};
use crate::StableOwned;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomPinned;
use core::mem::MaybeUninit;
use core::pin::Pin;

pub struct PinnedSelfie<'a, T: 'a, R: for<'this> RefType<'this> + ?Sized> {
    referential: MaybeUninit<<R as RefType<'a>>::Ref>,
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
            referential: MaybeUninit::uninit(),
            _pinned: PhantomPinned,
        };

        let mut pinned = P::new_pinned(this);

        // SAFETY: we are not moving owned, only initializing referential
        let pinned_mut: &mut Self = unsafe { Pin::get_unchecked_mut(P::pin_as_mut(&mut pinned)) };
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(unsafe { detach_lifetime_ref(&pinned_mut.owned) });

        pinned_mut.referential.write(referential);

        pinned
    }

    #[inline]
    pub fn owned(&self) -> &T {
        &self.owned
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        // SAFETY: referential was initialized in new_in_with
        // There is no safe way to get a PinnedSelfie without completing new_in_with
        unsafe { self.referential.assume_init_ref() }
    }

    #[inline]
    pub fn into_inner<P: StableOwned<Self>>(mut this: Pin<P>) -> T {
        // First, deallocate the referential before moving anything out
        // SAFETY: we are not moving anything… yet
        let selfie: &mut Self = unsafe { Pin::get_unchecked_mut(P::pin_as_mut(&mut this)) };

        // SAFETY: this essentially takes Self therefore can only be called once. Referential is
        // never accessed again after this, and is guaranteed to be initialized by the constructor
        unsafe { ::core::ptr::drop_in_place(selfie.referential.as_mut_ptr()) };

        // SAFETY: T is Unpin, and PinnedSelfie without the referential is inherently Unpin as well
        P::unwrap(unsafe { Pin::into_inner_unchecked(this) }).owned // TODO: check in case of panic
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
    referential: MaybeUninit<<R as RefType<'a>>::Ref>,
    owned: T,
    _pinned: PhantomPinned,
}

impl<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> PinnedSelfieMut<'a, T, R> {
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub fn new<P: 'a + StableOwned<Self>>(
        owned: T,
        handler: for<'this> fn(Pin<&'this mut T>) -> <R as RefType<'this>>::Ref,
    ) -> Pin<P> {
        let this = Self {
            owned,
            referential: MaybeUninit::uninit(),
            _pinned: PhantomPinned,
        };

        let mut pinned = P::new_pinned(this);

        // SAFETY: we are not moving owned, only initializing referential
        let pinned_mut = unsafe { Pin::get_unchecked_mut(P::pin_as_mut(&mut pinned)) };
        let (pinned_owned, pinned_referential) =
            (Pin::new(&mut pinned_mut.owned), &mut pinned_mut.referential);

        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(unsafe { detach_lifetime_mut(pinned_owned) });
        pinned_referential.write(referential);

        pinned
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        // SAFETY: referential was initialized in new_in_with
        // There is no safe way to get a PinnedSelfie without completing new_in_with
        unsafe { self.referential.assume_init_ref() }
    }

    #[inline]
    pub fn referential_mut(self: Pin<&mut Self>) -> &mut <R as RefType<'a>>::Ref {
        // SAFETY: the referential type is not structural for PinnedSelfieMut
        // SAFETY: referential was initialized in new_in_with
        // There is no safe way to get a PinnedSelfie without completing new_in_with
        unsafe { self.get_unchecked_mut().referential.assume_init_mut() }
    }

    #[inline]
    pub fn into_inner<P: StableOwned<Self>>(mut this: Pin<P>) -> T {
        // First, deallocate the referential before moving anything out
        // SAFETY: we are not moving anything… yet
        let selfie: &mut Self = unsafe { Pin::get_unchecked_mut(P::pin_as_mut(&mut this)) };

        // SAFETY: this essentially takes Self therefore can only be called once. Referential is
        // never accessed again after this, and is guaranteed to be initialized by the constructor
        unsafe { ::core::ptr::drop_in_place(selfie.referential.as_mut_ptr()) };

        // SAFETY: T is Unpin, and PinnedSelfie without the referential is inherently Unpin as well
        P::unwrap(unsafe { Pin::into_inner_unchecked(this) }).owned // TODO: check in case of panic
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
