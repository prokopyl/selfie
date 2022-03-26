//! Safe implementations for Selfie and SelfieMut that do not rely on anything internal to it

use crate::refs::*;
use crate::{Selfie, SelfieMut};
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;
use stable_deref_trait::StableDeref;

impl<'a, P: 'a + StableDeref, R: for<'this> RefType<'this> + ?Sized> Debug for Selfie<'a, P, R>
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

impl<'a, P: StableDeref + DerefMut + 'a, R: for<'this> RefType<'this> + ?Sized> Debug
    for SelfieMut<'a, P, R>
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
