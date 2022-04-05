use crate::refs::RefType;
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

pub trait IntoReferential<P, R>
where
    P: StableDeref,
    R: for<'this> RefType<'this>,
{
    fn into_referential(self, owned: &P::Target) -> <R as RefType>::Ref;
}

impl<P: StableDeref, R: for<'this> RefType<'this>> IntoReferential<P, R>
    for fn(&P::Target) -> <R as RefType>::Ref
{
    #[inline]
    fn into_referential(self, owned: &P::Target) -> <R as RefType>::Ref {
        self(owned)
    }
}

pub trait IntoReferentialMut<P, R>
where
    P: StableDeref + DerefMut,
    R: for<'this> RefType<'this>,
{
    fn into_referential(self, owned: Pin<&mut P::Target>) -> <R as RefType>::Ref;
}

impl<P: StableDeref + DerefMut, R: for<'this> RefType<'this>> IntoReferentialMut<P, R>
    for fn(Pin<&mut P::Target>) -> <R as RefType>::Ref
{
    #[inline]
    fn into_referential(self, owned: Pin<&mut P::Target>) -> <R as RefType>::Ref {
        self(owned)
    }
}
