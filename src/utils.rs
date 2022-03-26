use crate::refs::RefType;

#[inline]
pub unsafe fn detach_lifetime<'this, T: ?Sized>(
    pin: core::pin::Pin<&T>,
) -> core::pin::Pin<&'this T> {
    // SAFETY: wa are transmuting between the exact same types, except for the lifetimes, which
    // invariants are upheld by the caller
    ::core::mem::transmute(pin)
}

/// Same as detach_borrow but mut
#[inline]
pub unsafe fn detach_lifetime_mut<'this, T: ?Sized>(
    pin: core::pin::Pin<&mut T>,
) -> core::pin::Pin<&'this mut T> {
    // SAFETY: same as detach_borrow but mut
    ::core::mem::transmute(pin)
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
