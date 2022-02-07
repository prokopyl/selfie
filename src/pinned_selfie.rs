use crate::refs::*;
use core::mem::MaybeUninit;
use core::ops::DerefMut;
use core::pin::Pin;

pub struct PinnedSelfie<T, R: for<'a> RefType<'a> + ?Sized> {
    referential: MaybeUninit<<R as RefType<'static>>::Ref>,
    owned: T,
}

impl<T: 'static + Unpin, R: for<'a> RefType<'a> + ?Sized> PinnedSelfie<T, R> {
    #[inline]
    pub fn new_in<P: DerefMut<Target = Self>, F: FnOnce(Self) -> Pin<P>>(
        owned: T,
        pinner: F,
        handler: for<'a> fn(&'a T) -> <R as RefType<'a>>::Ref,
    ) -> Pin<P> {
        Self::new_in_with(owned, pinner, |p| p.as_mut(), handler)
    }

    // TODO: bikeshed
    pub fn new_in_with<P, F: FnOnce(Self) -> Pin<P>, FM: FnOnce(&mut Pin<P>) -> Pin<&mut Self>>(
        owned: T,
        pinner: F,
        as_mut: FM,
        handler: for<'a> fn(&'a T) -> <R as RefType<'a>>::Ref,
    ) -> Pin<P> {
        let this = Self {
            owned,
            referential: MaybeUninit::uninit(),
        };

        let mut pinned = pinner(this);

        // SAFETY: we are not moving owned, only initializing referential
        let pinned_mut: &mut Self = unsafe { Pin::get_unchecked_mut(as_mut(&mut pinned)) };
        // SAFETY: owned is pinned and cannot move, and this struct guarantees its lifetime
        let referential = handler(unsafe { &*(&pinned_mut.owned as *const _) });

        pinned_mut.referential.write(referential);

        pinned
    }

    #[inline]
    pub fn owned(&self) -> &T {
        &self.owned
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType>::Ref {
        // TODO: check this is actually safe
        unsafe { ::core::mem::transmute(&self.referential) }
    }

    #[inline]
    pub fn into_inner<P: Into<Self> + DerefMut<Target = Self>>(mut this: Pin<P>) -> T {
        // First, deallocate the referential before moving anything out
        // SAFETY: we are not moving anything… yet
        let selfie: &mut Self = unsafe { Pin::get_unchecked_mut(this.as_mut()) };

        // SAFETY: this essentially takes Self therefore can only be called once. Referential is
        // never accessed again after this, and is guaranteed to be intialized by the constructor
        unsafe { ::core::ptr::drop_in_place(selfie.referential.as_mut_ptr()) };

        // SAFETY: T is Unpin, and PinnedSelfie without referential is inherently Unpin as well
        unsafe { Pin::into_inner_unchecked(this) }.into().owned
    }
}
