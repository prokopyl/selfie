use core::ops::Deref;
use core::pin::Pin;
use stable_deref_trait::StableDeref;

pub unsafe trait StableOwned<T>: Sized + Deref<Target = T> + StableDeref {
    fn new_pinned(data: T) -> Pin<Self>;
    fn pin_as_mut(pin: &mut Pin<Self>) -> Pin<&mut T>;
    fn unwrap(self) -> T;
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
    #[cfg(feature = "std")]
    use std::{rc::Rc, sync::Arc};
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    extern crate alloc;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::{boxed::Box, rc::Rc, sync::Arc};

    unsafe impl<T> StableOwned<T> for Box<T> {
        #[inline]
        fn new_pinned(data: T) -> Pin<Self> {
            Box::pin(data)
        }

        #[inline]
        fn pin_as_mut(pin: &mut Pin<Self>) -> Pin<&mut T> {
            Pin::as_mut(pin)
        }

        #[inline]
        fn unwrap(self) -> T {
            *self
        }
    }

    unsafe impl<T> StableOwned<T> for Rc<T> {
        #[inline]
        fn new_pinned(data: T) -> Pin<Self> {
            Rc::pin(data)
        }

        #[inline]
        fn pin_as_mut(pin: &mut Pin<Self>) -> Pin<&mut T> {
            rc_get_pin_mut(pin).unwrap()
        }

        #[inline]
        fn unwrap(self) -> T {
            match Rc::try_unwrap(self) {
                Ok(value) => value,
                Err(_) => panic!("Failed to unwrap: Rc is still shared"),
            }
        }
    }

    unsafe impl<T> StableOwned<T> for Arc<T> {
        #[inline]
        fn new_pinned(data: T) -> Pin<Self> {
            Arc::pin(data)
        }

        #[inline]
        fn pin_as_mut(pin: &mut Pin<Self>) -> Pin<&mut T> {
            arc_get_pin_mut(pin).unwrap()
        }

        #[inline]
        fn unwrap(self) -> T {
            match Arc::try_unwrap(self) {
                Ok(value) => value,
                Err(_) => panic!("Failed to unwrap: Arc is still shared"),
            }
        }
    }

    #[inline]
    unsafe fn pin_get_ptr_unchecked_mut<P>(pin: &mut Pin<P>) -> &mut P {
        // SAFETY: Pin is repr(transparent). The caller is responsible to ensure the data will never be
        // moved, similar to Pin::get_unchecked_mut
        &mut *(pin as *mut _ as *mut P)
    }

    #[inline]
    fn rc_get_pin_mut<T>(pin: &mut Pin<Rc<T>>) -> Option<Pin<&mut T>> {
        // SAFETY: Arc::get_mut does not move anything
        let rc = unsafe { pin_get_ptr_unchecked_mut(pin) };
        let inner = Rc::get_mut(rc)?;

        // SAFETY: By using get_mut we guaranteed this is the only reference to it.
        // The &mut Pin<Arc<T>> argument guarantees this data was pinned, and the temporary Arc reference
        // is never exposed.
        unsafe { Some(Pin::new_unchecked(inner)) }
    }

    #[inline]
    fn arc_get_pin_mut<T>(pin: &mut Pin<Arc<T>>) -> Option<Pin<&mut T>> {
        // SAFETY: Arc::get_mut does not move anything
        let arc = unsafe { pin_get_ptr_unchecked_mut(pin) };
        let inner = Arc::get_mut(arc)?;

        // SAFETY: By using get_mut we guaranteed this is the only reference to it.
        // The &mut Pin<Arc<T>> argument guarantees this data was pinned, and the temporary Arc reference
        // is never exposed.
        unsafe { Some(Pin::new_unchecked(inner)) }
    }
};
