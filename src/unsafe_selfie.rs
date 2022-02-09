use crate::refs::RefType;
use stable_deref_trait::StableDeref;
use std::marker::PhantomPinned;
use std::ops::DerefMut;
use std::pin::Pin;

use crate::utils::*;

pub struct UnsafeSelfie<'a, P: 'a, R: for<'this> RefType<'this> + ?Sized> {
    // SAFETY: enforce drop order!
    referential: <R as RefType<'a>>::Ref,
    owned: Pin<P>,
}

impl<'a, P: StableDeref + 'a, R: for<'this> RefType<'this> + ?Sized> UnsafeSelfie<'a, P, R> {
    #[inline]
    pub fn new_ref(
        owned: Pin<P>,
        handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(unsafe { detach_lifetime(owned.as_ref()) }.get_ref());

        Self { referential, owned }
    }

    #[inline]
    pub fn new_mut(
        mut owned: Pin<P>,
        handler: for<'this> fn(Pin<&'this mut P::Target>) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
        P: DerefMut,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(unsafe { detach_lifetime_mut(owned.as_mut()) });

        Self { referential, owned }
    }

    /// # Safety
    ///
    /// The caller *must* ensure that `referential`'s shared lifetime restrictions are not violated.
    /// This means:
    ///
    /// * The `UnsafeSelfie` *must* have been created by shared reference using `new_ref`
    /// * The `referential_mut` method *must never* be called to retrieve an exclusive reference to
    ///   the referential (TODO: does it?)
    #[inline]
    pub unsafe fn owned(&self) -> &P::Target {
        self.owned.as_ref().get_ref()
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType<'a>>::Ref {
        &self.referential
    }

    #[inline]
    pub unsafe fn referential_mut(&mut self) -> &mut <R as RefType<'a>>::Ref {
        &mut self.referential
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.owned
    }
}

pub struct UnsafePinnedSelfie<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> {
    // SAFETY: enforce drop order!
    referential: UninitDrop<<R as RefType<'a>>::Ref>,
    owned: T,
    _pinned: PhantomPinned,
}

impl<'a, T: 'a + Unpin, R: for<'this> RefType<'this> + ?Sized + 'a> UnsafePinnedSelfie<'a, T, R> {
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub unsafe fn new_uninitialized(owned: T) -> Self {
        Self {
            owned,
            // SAFETY: TODO
            referential: UninitDrop::uninit(),
            _pinned: PhantomPinned,
        }
    }

    #[inline]
    pub unsafe fn write_referential(
        self: Pin<&mut Self>,
        handler: for<'this> fn(&'this T) -> <R as RefType<'this>>::Ref,
    ) {
        // SAFETY: we are not moving owned, only initializing referential
        let pinned_mut: &mut Self = Pin::get_unchecked_mut(self);
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(detach_lifetime_ref(&pinned_mut.owned));

        pinned_mut.referential.write(referential);
    }

    #[inline]
    pub unsafe fn write_referential_mut(
        self: Pin<&mut Self>,
        handler: for<'this> fn(&'this mut T) -> <R as RefType<'this>>::Ref,
    ) {
        // SAFETY: we are not moving owned, only initializing referential
        let pinned_mut: &mut Self = Pin::get_unchecked_mut(self);
        let (pinned_owned, pinned_referential) =
            (&mut pinned_mut.owned, &mut pinned_mut.referential);

        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let referential = handler(detach_lifetime_ref_mut(pinned_owned));

        pinned_referential.write(referential);
    }

    /// # Safety
    ///
    /// The caller *must* ensure that `referential`'s shared lifetime restrictions are not violated.
    /// This means:
    ///
    /// * The `UnsafePinnedSelfie` *must* have been created by shared reference using `new_ref`
    /// * The `referential_mut` method *must never* be called to retrieve an exclusive reference to
    ///   the referential (TODO: does it?)
    #[inline]
    pub unsafe fn owned(&self) -> &T {
        &self.owned
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

    /// # Safety
    ///
    /// This can only be called once, after which the `referential` must *never be accessed again*
    /// After this is called, this struct *must never* be dropped: the destructor will attempt to
    /// drop
    #[inline]
    pub unsafe fn drop_referential(self: Pin<&mut Self>) {
        // SAFETY: owned is not moved by this operation
        let selfie: &mut Self = self.get_unchecked_mut();

        // SAFETY: It is guaranteed by the caller than this will only be called once
        // It is guaranteed by the caller of new_uninitialized than referential has actually been initialized
        selfie.referential.drop_in_place();
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.referential.forget();
        // SAFETY: T is Unpin, and PinnedSelfie without the referential is inherently Unpin as well
        self.owned
    }
}
