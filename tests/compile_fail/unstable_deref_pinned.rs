use selfie::refs::Ref;
use selfie::PinnedSelfie;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

struct UnstableBox<T>(pub T);

impl<T> Deref for UnstableBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for UnstableBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn main() {
    let data: Pin<UnstableBox<PinnedSelfie<i32, Ref<i32>>>> = PinnedSelfie::new(42, |value| value);

    assert_eq!(42, *data.owned());
    assert_eq!(42, **data.referential());
    assert!(::core::ptr::eq(data.owned(), *data.referential()));

    // Moving obviously can't do much here, but still
    let data = Box::new(data);
    assert_eq!(42, *data.owned());
    assert_eq!(42, **data.referential());
    assert!(::core::ptr::eq(data.owned(), *data.referential()));

    drop(data);
}
