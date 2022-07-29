#![allow(unsafe_code)] // This module contains slightly-less-unsafe (but still unsafe) helpers.

use crate::refs::RefType;

#[inline]
pub unsafe fn detach_lifetime<'this, T: ?Sized>(reference: &T) -> &'this T {
    // SAFETY: wa are transmuting between the exact same types, except for the lifetimes, which
    // invariants are upheld by the caller
    ::core::mem::transmute(reference)
}

/// Same as detach_borrow but mut
#[inline]
pub unsafe fn detach_lifetime_mut<'this, T: ?Sized>(reference: &mut T) -> &'this mut T {
    // SAFETY: same as detach_borrow but mut
    ::core::mem::transmute(reference)
}

#[inline]
pub unsafe fn downcast_ref<'s, 'owned: 's, R: for<'this> RefType<'this> + ?Sized>(
    referential: &'s <R as RefType<'owned>>::Ref,
) -> &'s <R as RefType<'s>>::Ref {
    ::core::mem::transmute(referential)
}

#[inline]
pub unsafe fn downcast_mut<'s, 'owned: 's, R: for<'this> RefType<'this> + ?Sized>(
    referential: &'s mut <R as RefType<'owned>>::Ref,
) -> &'s mut <R as RefType<'s>>::Ref {
    ::core::mem::transmute(referential)
}
