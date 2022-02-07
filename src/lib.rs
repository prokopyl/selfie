#![no_std]

use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;

unsafe fn transmute_pin<T: ?Sized>(pin: Pin<&T>) -> Pin<&'static T> {
    ::core::mem::transmute(pin)
}

unsafe fn transmute_pin_mut<T: ?Sized>(pin: Pin<&mut T>) -> Pin<&'static mut T> {
    ::core::mem::transmute(pin)
}

pub trait RefType<'a> {
    type Ref: 'a + Sized;
}

pub struct Ref<T: ?Sized>(PhantomData<T>);
// TODO: 'static
impl<'a, T: 'a + ?Sized> RefType<'a> for Ref<T> {
    type Ref = &'a T;
}

pub struct Mut<T: ?Sized>(PhantomData<T>);

// TODO: 'static
impl<'a, T: 'a + ?Sized> RefType<'a> for Mut<T> {
    type Ref = &'a mut T;
}

pub struct Selfie<P, R: for<'a> RefType<'a> + ?Sized> {
    // SAFETY: enforce drop order!
    referential: <R as RefType<'static>>::Ref,
    pinned: Pin<P>,
}

impl<P: Deref, R: for<'a> RefType<'a> + ?Sized> Selfie<P, R> {
    pub fn new(
        pinned: Pin<P>,
        handler: for<'a> fn(&'a P::Target) -> <R as RefType<'a>>::Ref,
    ) -> Self
    where
        P::Target: 'static, // TODO
    {
        // SAFETY: derefd is pinned and cannot move, and this struct guarantees its lifetime
        let derefd = unsafe { transmute_pin(pinned.as_ref()) }.get_ref();

        let referential = handler(derefd);

        Self {
            pinned,
            referential,
        }
    }

    #[inline]
    pub fn owned(&self) -> &P::Target {
        self.pinned.as_ref().get_ref()
    }

    #[inline]
    pub fn referential(&self) -> &<R as RefType>::Ref {
        // TODO: check this is actually safe
        unsafe { ::core::mem::transmute(&self.referential) }
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.pinned
    }
}

pub struct SelfieMut<P, R: for<'a> RefType<'a> + ?Sized> {
    // SAFETY: enforce drop order!
    referential: <R as RefType<'static>>::Ref,
    pinned: Pin<P>,
}

impl<P: DerefMut, R: for<'a> RefType<'a> + ?Sized> SelfieMut<P, R> {
    pub fn new(
        mut pinned: Pin<P>,
        handler: for<'a> fn(Pin<&'a mut P::Target>) -> <R as RefType<'a>>::Ref,
    ) -> Self
    where
        P::Target: 'static, // TODO
    {
        // SAFETY: derefd is pinned and cannot move, and this struct guarantees its lifetime
        let derefd = unsafe { transmute_pin_mut(pinned.as_mut()) };

        let referential = handler(derefd);

        Self {
            pinned,
            referential,
        }
    }

    pub fn referential(&self) -> &<R as RefType>::Ref {
        // TODO: check this is actually safe
        unsafe { ::core::mem::transmute(&self.referential) }
    }

    pub fn referential_mut(&mut self) -> &mut <R as RefType>::Ref {
        // TODO: check this is actually safe
        unsafe { ::core::mem::transmute(&mut self.referential) }
    }

    #[inline]
    pub fn into_inner(self) -> Pin<P> {
        self.pinned
    }
}

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
