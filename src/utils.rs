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

/// Same as detach_borrow
#[inline]
pub unsafe fn detach_lifetime_ref<'this, T>(this: &T) -> &'this T {
    // SAFETY: same as detach_borrow
    &*(this as *const _)
}
