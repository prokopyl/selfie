use std::marker::PhantomData;
use std::ops::Deref;

pub trait RefType<'a> {
    type Ref: 'a + Sized;
}

pub struct Ref<T: ?Sized>(PhantomData<T>);

impl<'a, T: 'a + ?Sized> RefType<'a> for Ref<T> {
    type Ref = &'a T;
}

pub struct Selfie<P, R> {
    i: PhantomData<(P, R)>,
}

impl<P, R> Selfie<P, R>
where
    P: Deref,
    R: for<'this> RefType<'this>,
{
    #[inline]
    pub fn new(owned: P, handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref) {
        Self::new_with(owned, handler)
    }

    pub fn new_with<F>(owned: P, handler: F)
    where
        F: IntoReferential<P, R>,
    {
        let ptr = owned.deref();
        let _ = handler.into_referential(ptr);
    }
}

pub trait IntoReferential<P, R>
where
    P: Deref,
    R: for<'this> RefType<'this>,
{
    fn into_referential(self, owned: &P::Target) -> <R as RefType>::Ref;
}

impl<P: Deref, R: for<'this> RefType<'this>> IntoReferential<P, R>
    for fn(&P::Target) -> <R as RefType>::Ref
{
    #[inline]
    fn into_referential(self, owned: &P::Target) -> <R as RefType>::Ref {
        self(owned)
    }
}

pub fn example() {
    let _ = Selfie::<Box<i32>, Ref<i32>>::new(Box::new(42), |i| i);
}
