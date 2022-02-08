use selfie::refs::Ref;
use selfie::PinnedSelfie;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

#[test]
pub fn pinned() {
    let data: Pin<Box<PinnedSelfie<i32, Ref<i32>>>> = PinnedSelfie::new(42, |value| value);

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

#[test]
pub fn pinned_rc() {
    let data: Pin<Rc<PinnedSelfie<i32, Ref<i32>>>> = PinnedSelfie::new(42, |value| value);

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

#[test]
pub fn pinned_arc() {
    let data: Pin<Arc<PinnedSelfie<i32, Ref<i32>>>> = PinnedSelfie::new(42, |value| value);

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
