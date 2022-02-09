use std::mem::MaybeUninit;

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

/// Same as detach_borrow
#[inline]
pub unsafe fn detach_lifetime_ref<'this, T>(this: &T) -> &'this T {
    // SAFETY: same as detach_borrow
    &*(this as *const _)
}

/// Same as detach_borrow
#[inline]
pub unsafe fn detach_lifetime_ref_mut<'this, T>(this: &mut T) -> &'this mut T {
    // SAFETY: same as detach_borrow
    &mut *(this as *mut _)
}

pub struct UninitDrop<T> {
    inner: MaybeUninit<T>,
}

impl<T> UninitDrop<T> {
    /// # Safety
    /// This must *not* be dropped if uninitialized
    #[inline]
    pub const unsafe fn uninit() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }

    #[inline]
    pub fn write(&mut self, value: T) -> &mut T {
        self.inner.write(value)
    }

    #[inline]
    pub unsafe fn assume_init_ref(&self) -> &T {
        self.inner.assume_init_ref()
    }

    #[inline]
    pub unsafe fn assume_init_mut(&mut self) -> &mut T {
        self.inner.assume_init_mut()
    }

    #[inline]
    pub fn forget(self) {
        ::core::mem::forget(self)
    }

    #[inline]
    pub unsafe fn drop_in_place(&mut self) {
        ::core::ptr::drop_in_place(self.inner.as_mut_ptr())
    }
}

impl<T> Drop for UninitDrop<T> {
    fn drop(&mut self) {
        // SAFETY: TODO
        unsafe { self.drop_in_place() }
    }
}
