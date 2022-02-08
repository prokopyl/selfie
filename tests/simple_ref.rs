use selfie::refs::{Ref, RefType};
use selfie::{PinnedSelfie, Selfie, SelfieMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

#[test]
pub fn simple_int() {
    let my_int = Box::pin(42);
    let data: Selfie<Box<i32>, Ref<i32>> = Selfie::new(my_int, |i| i);

    assert_eq!(42, *data.owned());
    assert_eq!(&42, *data.referential());

    let data = Box::new(data);

    assert_eq!(42, *data.owned());
    assert_eq!(&42, *data.referential());
}

#[test]
pub fn simple_str() {
    let my_str = Pin::new("Hello, world!".to_owned());
    let data: Selfie<String, Ref<str>> = Selfie::new(my_str, |i| &i[0..5]);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!(&"Hello", data.referential());

    let data = Box::new(data);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!(&"Hello", data.referential());
}

#[test]
pub fn different_int() {
    let my_int = Arc::pin(42);
    let data: Selfie<Arc<i32>, Ref<i32>> = Selfie::new(my_int, |i| i);

    assert_eq!(42, *data.owned());
    assert_eq!(&42, *data.referential());

    let data = Box::new(data);

    assert_eq!(42, *data.owned());
    assert_eq!(&42, *data.referential());
}

struct Point<'a> {
    x: &'a mut i32,
    y: &'a mut i32,
}

impl<'a> Point<'a> {
    fn new(values: Pin<&'a mut (i32, i32)>) -> Self {
        let values = Pin::into_inner(values);
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
pub fn struct_mut() {
    let my_str = Box::pin((0, 42));
    let mut data: SelfieMut<Box<(i32, i32)>, PointMut> = SelfieMut::new(my_str, |i| Point::new(i));

    assert_eq!(0, *data.referential().x);
    assert_eq!(42, *data.referential().y);
    *data.referential_mut().x = 69;
    assert_eq!(69, *data.referential().x);
    assert_eq!(42, *data.referential().y);

    let mut data = Box::new(data);

    assert_eq!(69, *data.referential().x);
    assert_eq!(42, *data.referential().y);
    *data.referential_mut().x = 12;
    assert_eq!(12, *data.referential().x);
    assert_eq!(42, *data.referential().y);
}

struct Dropper<'a> {
    value: &'a str,
}

impl<'a> Drop for Dropper<'a> {
    fn drop(&mut self) {
        assert_eq!("Hello", self.value)
    }
}

struct DropperRef;

impl<'a> RefType<'a> for DropperRef {
    type Ref = Dropper<'a>;
}

#[test]
pub fn drops() {
    let my_str = Pin::new("Hello".to_owned().into_boxed_str());
    let data: Selfie<Box<str>, DropperRef> = Selfie::new(my_str, |value| Dropper { value });

    assert_eq!("Hello", data.owned());
    assert_eq!("Hello", data.referential().value);

    let data = Box::new(data);
    assert_eq!("Hello", data.owned());
    assert_eq!("Hello", data.referential().value);

    drop(data);
}

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

fn all_but_first_char(x: &RefCell<String>) -> Selfie<::core::cell::Ref<String>, Ref<str>> {
    let x = Pin::new(x.borrow());
    Selfie::new(x, |s| &s[1..])
}

#[test]
pub fn refcell() {
    let refcell = RefCell::new("Hello, world!".to_owned());

    let selfie = all_but_first_char(&refcell);
    assert!(refcell.try_borrow_mut().is_err());
    assert_eq!("ello, world!", *selfie.referential());
    drop(selfie);

    assert!(refcell.try_borrow_mut().is_ok());
}

struct UnstableInt(pub i32);

impl Deref for UnstableInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[test] // FIXME: this should fail
pub fn unpinned_int() {
    let int = Pin::new(UnstableInt(42));
    let data: Selfie<UnstableInt, Ref<i32>> = Selfie::new(int, |i| i);

    assert_eq!(&42, *data.referential());

    let data = Box::new(data);
    assert_eq!(&42, *data.referential());
}
