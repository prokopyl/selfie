use crate::{Selfie, SelfieMut};
use core::marker::PhantomData;

pub trait RefType<'a> {
    type Ref: 'a + Sized;
}

pub struct Ref<T: ?Sized>(PhantomData<T>);

impl<'a, T: 'a + ?Sized> RefType<'a> for Ref<T> {
    type Ref = &'a T;
}

pub struct Mut<T: ?Sized>(PhantomData<T>);

impl<'a, T: 'a + ?Sized> RefType<'a> for Mut<T> {
    type Ref = &'a mut T;
}

// TODO: bikeshed all these
pub struct SelfieRef<P, R>(PhantomData<P>, PhantomData<R>)
where
    P: ?Sized,
    R: ?Sized;

impl<'a, P, R> RefType<'a> for SelfieRef<P, R>
where
    P: RefType<'a>,
    R: 'a + for<'this> RefType<'this>,
{
    type Ref = Selfie<'a, P::Ref, R>;
}

pub struct SelfieRefMut<P, R>(PhantomData<P>, PhantomData<R>)
where
    P: ?Sized,
    R: ?Sized;

impl<'a, P, R> RefType<'a> for SelfieRefMut<P, R>
where
    P: RefType<'a>,
    R: 'a + for<'this> RefType<'this> + ?Sized,
{
    type Ref = SelfieMut<'a, P::Ref, R>;
}
