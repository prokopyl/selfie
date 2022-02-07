use selfie::{Ref, RefType, Selfie, SelfieMut};
use std::pin::Pin;

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
    let my_str = Pin::new("Hello, world!".to_owned().into_boxed_str());
    let data: Selfie<Box<str>, Ref<str>> = Selfie::new(my_str, |i| &i[0..5]);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!(&"Hello", data.referential());

    let val = data;
    let data = Box::new(val);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!(&"Hello", data.referential());
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
