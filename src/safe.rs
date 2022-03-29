//! Safe implementations for Selfie and SelfieMut that do not rely on anything internal to it

use crate::convert::ToReferential;
use crate::refs::*;
use crate::{PinnedSelfie, PinnedSelfieMut, Selfie, SelfieMut};
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

impl<'a, P, R> Selfie<'a, P, R>
where
    P: StableDeref + 'a,
    R: for<'this> RefType<'this>,
    P::Target: 'a,
{
    #[inline]
    pub fn new(
        owned: Pin<P>,
        handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
    ) -> Self {
        struct FnToReferential<P: StableDeref, R: for<'this> RefType<'this>>(
            for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
        );

        impl<P: StableDeref, R: for<'this> RefType<'this>> ToReferential<P, R> for FnToReferential<P, R> {
            #[inline]
            fn to_referential(self, owned: &P::Target) -> <R as RefType>::Ref {
                (self.0)(owned)
            }
        }

        Self::new_with(owned, FnToReferential(handler))
    }
}

impl<'a, P, R> Debug for Selfie<'a, P, R>
where
    P::Target: Debug,
    for<'this> <R as RefType<'this>>::Ref: Debug,
    P: 'a + StableDeref,
    R: for<'this> RefType<'this>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("Selfie")
                .field("owned", &self.owned())
                .field("referential", referential)
                .finish()
        })
    }
}

impl<'a, P, R> Debug for SelfieMut<'a, P, R>
where
    for<'this> <R as RefType<'this>>::Ref: Debug,
    P: StableDeref + DerefMut + 'a,
    R: for<'this> RefType<'this>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("SelfieMut")
                .field("referential", referential)
                .finish()
        })
    }
}

impl<'a, P, R> Debug for PinnedSelfie<'a, P, R>
where
    for<'this> <R as RefType<'this>>::Ref: Debug,
    P: 'a + Debug,
    R: for<'this> RefType<'this>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("PinnedSelfie")
                .field("owned", &self.owned())
                .field("referential", referential)
                .finish()
        })
    }
}
/*
impl<'a, P, R> Debug for PinnedSelfieMut<'a, P, R>
where
    for<'this> <R as RefType<'this>>::Ref: Debug,
    P: StableDeref + DerefMut + 'a,
    R: for<'this> RefType<'this>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("PinnedSelfieMut")
                .field("referential", referential)
                .finish()
        })
    }
}
*/
