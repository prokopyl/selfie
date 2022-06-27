//! Safe implementations for Selfie and SelfieMut that do not rely on anything internal to it

use crate::refs::*;
use crate::{Selfie, SelfieMut};
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use stable_deref_trait::StableDeref;

impl<'a, P, R> Selfie<'a, P, R>
where
    P: StableDeref + 'a,
    R: for<'this> RefType<'this>,
    P::Target: 'a,
{
    #[inline]
    pub fn referential<'s>(&'s self) -> <R as RefType<'s>>::Ref
    where
        <R as RefType<'s>>::Ref: Copy,
    {
        self.with_referential(|r| *r)
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

impl<'a, P, R> SelfieMut<'a, P, R>
where
    P: StableDeref + DerefMut + 'a,
    R: for<'this> RefType<'this>,
    P::Target: 'a,
{
    #[inline]
    pub fn referential<'s>(&'s self) -> <R as RefType<'s>>::Ref
    where
        <R as RefType<'s>>::Ref: Copy,
    {
        self.with_referential(|r| *r)
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
            f.debug_struct("Selfie")
                .field("referential", referential)
                .finish()
        })
    }
}
