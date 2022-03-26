use crate::refs::RefType;
use stable_deref_trait::StableDeref;

pub trait ToReferential<P: StableDeref, R: for<'this> RefType<'this>> {
    fn to_referential(self, owned: &P::Target) -> <R as RefType>::Ref;
}

impl<P: StableDeref, R: for<'this> RefType<'this>> ToReferential<P, R>
    for fn(&P::Target) -> <R as RefType>::Ref
{
    #[inline]
    fn to_referential(self, owned: &P::Target) -> <R as RefType>::Ref {
        self(owned)
    }
}
