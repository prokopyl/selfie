#![allow(unsafe_code)]

use crate::refs::*;
use crate::utils::*;
use crate::StableOwned;
use core::marker::PhantomPinned;
use core::pin::Pin;
use std::fmt::Debug;

pub struct PinnedSelfie<'a, O, R>
where
    O: 'a,
    R: for<'this> RefType<'this>,
{
    referential: Option<<R as RefType<'a>>::Ref>,
    owned: O,
    _pinned: PhantomPinned,
}

impl<'a, O, R> PinnedSelfie<'a, O, R>
where
    O: 'a,
    R: for<'this> RefType<'this>,
{
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub fn new<P: 'a + StableOwned<Self>>(
        owned: O,
        handler: for<'this> fn(&'this O) -> <R as RefType<'this>>::Ref,
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
    pub fn owned(&self) -> &O {
        &self.owned
    }

    #[inline]
    pub fn with_referential<'s, F, T>(&'s self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this <R as RefType<'s>>::Ref) -> T,
    {
        let referential = self.referential.as_ref().unwrap();
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(referential) };
        handler(referential)
    }

    #[inline]
    pub fn with_referential_mut<'s, F, T>(self: Pin<&'s mut Self>, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this mut <R as RefType<'s>>::Ref) -> T,
        O: Copy + Debug,
    {
        // SAFETY: the referential type is not structural for the selfie, only the owned part is
        let unpinned = unsafe { self.get_unchecked_mut() };
        let referential = unpinned.referential.as_mut().unwrap();
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_mut::<'s, 'a, R>(referential) };

        handler(referential)
    }
}

pub struct PinnedSelfieMut<'a, O, R>
where
    O: 'a,
    R: for<'this> RefType<'this>,
{
    referential: Option<<R as RefType<'a>>::Ref>,
    owned: O,
    _pinned: PhantomPinned,
}

impl<'a, O, R> PinnedSelfieMut<'a, O, R>
where
    O: 'a,
    R: for<'this> RefType<'this>,
{
    #[inline]
    #[allow(clippy::new_ret_no_self)] // It actually returns Self, but Clippy can't really know that
    pub fn new<P: 'a + StableOwned<Self>>(
        owned: O,
        handler: for<'this> fn(&'this mut O) -> <R as RefType<'this>>::Ref,
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
