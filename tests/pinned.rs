use selfie::refs::Ref;
use selfie::PinnedSelfie;
use std::pin::Pin;

#[test]
pub fn pinned() {
    let data: Pin<Box<PinnedSelfie<i32, Ref<i32>>>> =
        PinnedSelfie::new_in(42, Box::pin, |value| value);

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
