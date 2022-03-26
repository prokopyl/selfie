//! Safe implementations for Selfie and SelfieMut that do not rely on anything internal to it

use crate::convert::ToReferential;
use crate::refs::*;
use crate::{Selfie, SelfieMut};
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

impl<'a, P: StableDeref + 'a, R: for<'this> RefType<'this>> Selfie<'a, P, R> {
    #[inline]
    pub fn new(
        owned: Pin<P>,
        handler: for<'this> fn(&'this P::Target) -> <R as RefType<'this>>::Ref,
    ) -> Self
    where
        P::Target: 'a,
    {
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

impl<'a, P: 'a + StableDeref, R: for<'this> RefType<'this>> Debug for Selfie<'a, P, R>
where
    P::Target: Debug,
    for<'this> <R as RefType<'this>>::Ref: Debug,
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

impl<'a, P: StableDeref + DerefMut + 'a, R: for<'this> RefType<'this>> Debug for SelfieMut<'a, P, R>
where
    for<'this> <R as RefType<'this>>::Ref: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.with_referential(|referential| {
            f.debug_struct("Selfie")
                .field("referential", referential)
                .finish()
        })
    }
}
