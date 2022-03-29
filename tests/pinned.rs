use selfie::refs::{Ref, RefType};
use selfie::{PinnedSelfie, PinnedSelfieMut};
use std::cell::Cell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

#[test]
pub fn pinned() {
    let mut data: Pin<Box<PinnedSelfie<i32, Ref<i32>>>> = PinnedSelfie::new(42, |value| value);

    assert_eq!(42, *data.owned());
    assert_eq!(42, data.with_referential(|r| **r));
    // data.as_mut().with_referential_mut(|r| *r = &5);
    assert_eq!(5, data.as_mut().with_referential_mut(|r| **r));
    // assert!(::core::ptr::eq(data.owned(), data.with_referential(|r| *r)));

    // Moving obviously can't do much here, but still
    let data = Box::new(data);
    assert_eq!(42, *data.owned());
    assert_eq!(5, data.with_referential(|r| **r));
    // assert!(::core::ptr::eq(data.owned(), data.with_referential(|r| *r)));

    drop(data);
}

#[test]
pub fn pinned_rc() {
    let data: Pin<Rc<PinnedSelfie<i32, Ref<i32>>>> = PinnedSelfie::new(42, |value| value);

    assert_eq!(42, *data.owned());
    assert_eq!(42, data.with_referential(|r| **r));
    assert!(::core::ptr::eq(data.owned(), data.with_referential(|r| *r)));

    // Moving obviously can't do much here, but still
    let data = Box::new(data);
    assert_eq!(42, *data.owned());
    assert_eq!(42, data.with_referential(|r| **r));
    assert!(::core::ptr::eq(data.owned(), data.with_referential(|r| *r)));

    drop(data);
}

#[test]
pub fn pinned_arc() {
    let data: Pin<Arc<PinnedSelfie<i32, Ref<i32>>>> = PinnedSelfie::new(42, |value| value);

    assert_eq!(42, *data.owned());
    assert_eq!(42, data.with_referential(|r| **r));
    assert!(::core::ptr::eq(data.owned(), data.with_referential(|r| *r)));

    // Moving obviously can't do much here, but still
    let data = Box::new(data);
    assert_eq!(42, *data.owned());
    assert_eq!(42, data.with_referential(|r| **r));
    assert!(::core::ptr::eq(data.owned(), data.with_referential(|r| *r)));

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
/*
#[test]
pub fn pinned_mut() {
    let mut data: Pin<Box<PinnedSelfieMut<(i32, i32), PointMut>>> =
        PinnedSelfieMut::new((0, 42), |value| Point::new(value));

    assert_eq!(0, *data.with_referential(|r| r.x));
    assert_eq!(42, *data.with_referential(|r| r.y));
    *data.as_mut().referential_mut().x = 69;
    assert_eq!(69, data.with_referential(|r| *r).x);
    assert_eq!(42, data.with_referential(|r| *r).y);

    let mut data = Box::new(data);

    assert_eq!(69, data.with_referential(|r| *r).x);
    assert_eq!(42, data.with_referential(|r| *r).y);
    *data.as_mut().as_mut().referential_mut().x = 12;
    assert_eq!(12, data.with_referential(|r| *r).x);
    assert_eq!(42, data.with_referential(|r| *r).y);
}
*/
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

    assert_eq!(42, dropper.with_referential(|r| r.value.value));
    assert_eq!(42, dropper.owned().value);
    assert!(!dropper.owned().dropped.get());

    // Drop the ref
    drop(dropper);

    assert!(checker.dropped.get());
}
