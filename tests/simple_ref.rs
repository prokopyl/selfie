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
    data.with_referential(|i| assert_eq!(&42, *i));

    let data = Box::new(data);

    assert_eq!(42, *data.owned());
    data.with_referential(|i| assert_eq!(&42, *i));
}

#[test]
pub fn simple_str() {
    let my_str = Pin::new("Hello, world!".to_owned());
    let data: Selfie<String, Ref<str>> = Selfie::new(my_str, |i| &i[0..5]);

    assert_eq!("Hello, world!", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", i));

    let data = Box::new(data);

    assert_eq!("Hello, world!", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", i));
}

#[test]
pub fn different_int() {
    let my_int = Arc::pin(42);
    let data: Selfie<Arc<i32>, Ref<i32>> = Selfie::new(my_int, |i| i);

    assert_eq!(42, *data.owned());
    data.with_referential(|i| assert_eq!(&42, *i));

    let data = Box::new(data);

    assert_eq!(42, *data.owned());
    data.with_referential(|i| assert_eq!(&42, *i));
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

impl<'a> RefType<'a> for DropperRef {
    type Ref = Dropper<'a>;
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
    selfie.with_referential(|i| assert_eq!(&"ello, world!", i));
    drop(selfie);

    assert!(refcell.try_borrow_mut().is_ok());
}

struct Bar<'a>(RefCell<(Option<&'a Bar<'a>>, String)>);

struct BarRef;
impl<'a> RefType<'a> for BarRef {
    type Ref = Bar<'a>;
}

/*#[test]
fn main() {
    let value = Box::pin(());

    let mut selfie: Selfie<Box<()>, BarRef> =
        Selfie::new(value, |_| Bar(RefCell::new((None, "Hello".to_owned()))));

    selfie.referential().0.borrow_mut().0 = Some(selfie.referential());

    let dep = selfie.referential_mut();
    let r1 = dep.0.get_mut();
    let string_ref_1 = &mut r1.1;
    let mut r2 = r1.0.unwrap().0.borrow_mut();
    let string_ref_2 = &mut r2.1;

    let s = &string_ref_1[..];
    string_ref_2.clear();
    string_ref_2.shrink_to_fit();
    println!("{}", s); // prints garbage
}*/

/*#[test]
fn normal() {
    let mut bar: &mut Bar = Box::leak(Box::new(Bar(RefCell::new((None, "Hello".to_owned())))));

    bar.0.borrow_mut().0 = Some(&bar);

    let mut r1 = bar.0.get_mut();
    let string_ref_1 = &mut r1.1;
    let mut r2 = r1.0.unwrap().0.borrow_mut();
    let string_ref_2 = &mut r2.1;

    let s = &string_ref_1[..];
    string_ref_2.clear();
    string_ref_2.shrink_to_fit();
    println!("{}", s); // prints garbage
}
*/
