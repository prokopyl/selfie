#![allow(unsafe_code)] // This module contains slightly-less-unsafe (but still unsafe) helpers.

use crate::refs::RefType;
use core::pin::Pin;

#[inline]
pub unsafe fn detach_lifetime<'this, T: ?Sized>(pin: Pin<&T>) -> Pin<&'this T> {
    // SAFETY: wa are transmuting between the exact same types, except for the lifetimes, which
    // invariants are upheld by the caller
    ::core::mem::transmute(pin)
}

/// Same as detach_borrow but mut
#[inline]
pub unsafe fn detach_lifetime_mut<'this, T: ?Sized>(pin: Pin<&mut T>) -> Pin<&'this mut T> {
    // SAFETY: same as detach_borrow but mut
    ::core::mem::transmute(pin)
}

#[inline]
pub unsafe fn downcast_ref<'s, 'owned: 's, R: RefType + ?Sized>(
    referential: &'s R::Ref<'owned>,
) -> &'s R::Ref<'s> {
    ::core::mem::transmute(referential)
}

#[inline]
pub unsafe fn downcast_mut<'s, 'owned: 's, R: RefType + ?Sized>(
    referential: &'s mut R::Ref<'owned>,
) -> &'s mut R::Ref<'s> {
    ::core::mem::transmute(referential)
}
