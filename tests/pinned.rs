use selfie::refs::{Ref, RefType};
use selfie::{PinnedSelfie, PinnedSelfieMut};
use std::cell::Cell;
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

struct Point<'a> {
    x: &'a mut i32,
    y: &'a mut i32,
}

impl<'a> Point<'a> {
    fn new(values: &'a mut (i32, i32)) -> Self {
        Self {
            x: &mut values.0,
            y: &mut values.1,
        }
    }
}
struct PointMut;

impl<'a> RefType<'a> for PointMut {
    type Ref = Point<'a>;
}

#[test]
pub fn pinned_mut() {
    let mut data: Pin<Box<PinnedSelfieMut<(i32, i32), PointMut>>> =
        PinnedSelfieMut::new((0, 42), |value| Point::new(value));

    assert_eq!(0, *data.referential().x);
    assert_eq!(42, *data.referential().y);
    *data.as_mut().referential_mut().x = 69;
    assert_eq!(69, *data.referential().x);
    assert_eq!(42, *data.referential().y);

    let mut data = Box::new(data);

    assert_eq!(69, *data.referential().x);
    assert_eq!(42, *data.referential().y);
    *data.as_mut().as_mut().referential_mut().x = 12;
    assert_eq!(12, *data.referential().x);
    assert_eq!(42, *data.referential().y);
}

#[derive(Debug)]
struct DropChecker {
    value: u8,
    dropped: Cell<bool>,
}

struct Dropper<'a> {
    value: &'a &'a DropChecker,
}

impl<'a> Drop for Dropper<'a> {
    fn drop(&mut self) {
        assert!(!self.value.dropped.get());
        assert_eq!(42, self.value.value);
        self.value.dropped.set(true);
    }
}

struct DropperRef;

impl<'a> RefType<'a> for DropperRef {
    type Ref = Dropper<'a>;
}

#[test]
pub fn pinned_drop() {
    let checker = DropChecker {
        value: 42,
        dropped: Cell::new(false),
    };

    let dropper: Pin<Box<PinnedSelfie<&DropChecker, DropperRef>>> =
        PinnedSelfie::new(&checker, |value| Dropper { value });

    assert_eq!(42, dropper.referential().value.value);
    assert_eq!(42, dropper.owned().value);
    assert!(!dropper.owned().dropped.get());

    // Drop the ref
    drop(dropper);

    assert!(checker.dropped.get());
}

#[test]
pub fn pinned_drop_into_inner() {
    let checker = DropChecker {
        value: 42,
        dropped: Cell::new(false),
    };

    let dropper: Pin<Box<PinnedSelfie<&DropChecker, DropperRef>>> =
        PinnedSelfie::new(&checker, |value| Dropper { value });

    assert_eq!(42, dropper.referential().value.value);
    assert_eq!(42, dropper.owned().value);
    assert!(!dropper.owned().dropped.get());

    // Drop the ref
    let _ = PinnedSelfie::into_inner(dropper);

    assert!(checker.dropped.get());
}
