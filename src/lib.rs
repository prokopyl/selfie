use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;

pub struct RefHandle<T: ?Sized>(PhantomData<Box<T>>);

// TODO: 'static
impl<'a, T: 'a + ?Sized> PinHeaded<'a> for RefHandle<T> {
    type Ref = &'a T;
}

pub trait PinHeaded<'a> {
    type Ref: 'a + Sized;
}

pub struct PinHead<P: Deref, H: for<'a> PinHeaded<'a> + ?Sized> {
    pinned: Pin<P>,
    referential: <H as PinHeaded<'static>>::Ref,
}

impl<P: Deref, H: for<'a> PinHeaded<'a> + ?Sized> PinHead<P, H> {
    pub fn new(
        pinned: Pin<P>,
        handler: for<'a> fn(&'a P::Target) -> <H as PinHeaded<'a>>::Ref,
    ) -> Self
    where
        P::Target: 'static, // TODO
    {
        let derefd = pinned.as_ref().get_ref();

        // SAFETY: derefd is pinned and cannot move, and this struct guarantees its lifetime
        let derefd_static = unsafe { &*(derefd as *const _) };
        let referential = handler(derefd_static);

        Self {
            pinned,
            referential,
        }
    }

    #[inline]
    pub fn owned(&self) -> &P::Target {
        self.pinned.as_ref().get_ref()
    }

    pub fn referential(&self) -> &<H as PinHeaded>::Ref {
        // TODO: check this is actually safe
        unsafe { ::core::mem::transmute(&self.referential) }
    }
}
