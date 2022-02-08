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

#[cfg(feature = "stable_deref_trait")]
mod stable_deref_trait {
    use super::*;
    use crate::{Selfie, SelfieMut};

    // TODO: bikeshed all these
    pub struct SelfieRef<P: ?Sized, R: ?Sized>(PhantomData<P>, PhantomData<R>);

    impl<'a, P: RefType<'a>, R: 'a + for<'this> RefType<'this> + ?Sized> RefType<'a>
        for SelfieRef<P, R>
    {
        type Ref = Selfie<'a, P::Ref, R>;
    }

    pub struct SelfieRefMut<P: ?Sized, R: ?Sized>(PhantomData<P>, PhantomData<R>);

    impl<'a, P: RefType<'a>, R: 'a + for<'this> RefType<'this> + ?Sized> RefType<'a>
        for SelfieRefMut<P, R>
    {
        type Ref = SelfieMut<'a, P::Ref, R>;
    }
}

#[cfg(feature = "stable_deref_trait")]
pub use self::stable_deref_trait::*;
