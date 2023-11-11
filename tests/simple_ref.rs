use selfie::refs::{Ref, RefType};
use selfie::{Selfie, SelfieMut};
use std::cell::RefCell;
use std::pin::Pin;
use std::sync::Arc;

#[test]
pub fn simple_int() {
    let my_int = Box::pin(42);
    let data: Selfie<Box<i32>, Ref<i32>> = Selfie::new(my_int, |i| i);

    assert_eq!(42, *data.owned());
    assert_eq!(42, *data.with_referential(|r| *r));

    let data = Box::new(data);

    assert_eq!(42, *data.owned());
    assert_eq!(42, *data.with_referential(|r| *r));
}

#[test]
pub fn simple_str() {
    let my_str = Pin::new("Hello, world!".to_owned());
    let data: Selfie<String, Ref<str>> = Selfie::new(my_str, |i| &i[0..5]);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!("Hello", data.with_referential(|r| *r));

    let data = Box::new(data);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!("Hello", data.with_referential(|r| *r));
}

#[test]
pub fn different_int() {
    let my_int = Arc::pin(42);
    let data: Selfie<Arc<i32>, Ref<i32>> = Selfie::new(my_int, |i| i);

    assert_eq!(42, *data.owned());
    assert_eq!(42, *data.with_referential(|r| *r));

    let data = Box::new(data);

    assert_eq!(42, *data.owned());
    assert_eq!(42, *data.with_referential(|r| *r));
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

impl<'owned> RefType<'owned> for PointMut {
    type Ref<'a> = Point<'a> where 'owned: 'a;
}

#[test]
pub fn struct_mut() {
    let my_str = Box::pin((0, 42));
    let mut data: SelfieMut<Box<(i32, i32)>, PointMut> = SelfieMut::new(my_str, |i| Point::new(i));

    data.with_referential(|p| assert_eq!(0, *p.x));
    data.with_referential(|p| assert_eq!(42, *p.y));
    data.with_referential_mut(|p| *p.x = 69);
    data.with_referential(|p| assert_eq!(69, *p.x));
    data.with_referential(|p| assert_eq!(42, *p.y));

    let mut data = Box::new(data);

    data.with_referential(|p| assert_eq!(69, *p.x));
    data.with_referential(|p| assert_eq!(42, *p.y));
    data.with_referential_mut(|p| *p.x = 12);
    data.with_referential(|p| assert_eq!(12, *p.x));
    data.with_referential(|p| assert_eq!(42, *p.y));
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

impl<'owned> RefType<'owned> for DropperRef {
    type Ref<'a> = Dropper<'a> where 'owned: 'a;
}

#[test]
pub fn drops() {
    let my_str = Pin::new("Hello".to_owned().into_boxed_str());
    let data: Selfie<Box<str>, DropperRef> = Selfie::new(my_str, |value| Dropper { value });

    assert_eq!("Hello", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", &i.value));

    let data = Box::new(data);
    assert_eq!("Hello", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", &i.value));

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
    assert_eq!("ello, world!", selfie.with_referential(|r| *r));
    drop(selfie);

    assert!(refcell.try_borrow_mut().is_ok());
}
