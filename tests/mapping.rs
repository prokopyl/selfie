use selfie::refs::{Ref, RefType};
use selfie::Selfie;
use std::panic::catch_unwind;
use std::pin::Pin;
use std::rc::Rc;

#[test]
pub fn simple_map() {
    let data = Pin::new("Hello, world!".to_owned());
    let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |str| &str[0..5]);
    selfie.with_referential(|s| assert_eq!("Hello", *s));

    let selfie = selfie.map::<Ref<str>, _>(|str, _| &str[3..]);
    selfie.with_referential(|s| assert_eq!("lo", *s));

    let selfie: Selfie<String, Ref<str>> = selfie.map(|_, owned| &owned[7..]);
    selfie.with_referential(|s| assert_eq!("world!", *s));
}

#[derive(Debug)]
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
pub fn with_dropped_value() {
    let my_str = Pin::new("Hello".to_owned().into_boxed_str());
    let data: Selfie<Box<str>, DropperRef> = Selfie::new(my_str, |value| Dropper { value });

    assert_eq!("Hello", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", &i.value));

    let data: Selfie<Box<str>, DropperRef> = data.map(|dropper, _| Dropper {
        value: dropper.value,
    });
    assert_eq!("Hello", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", &i.value));
}

#[test]
pub fn panic_with_dropped_value() {
    let my_str = Pin::new("Hello".to_owned().into_boxed_str());
    let data: Selfie<Box<str>, DropperRef> = Selfie::new(my_str, |value| Dropper { value });

    assert_eq!("Hello", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", &i.value));

    // This should not lead to reading a dropped value or anything
    catch_unwind(|| {
        let _: Selfie<Box<str>, DropperRef> = data.map(|_, _| panic!("Haha"));
    })
    .unwrap_err();
}

#[test]
pub fn error_with_dropped_value() {
    let my_str = Pin::new("Hello".to_owned().into_boxed_str());
    let data: Selfie<Box<str>, DropperRef> = Selfie::new(my_str, |value| Dropper { value });

    assert_eq!("Hello", data.owned());
    data.with_referential(|i| assert_eq!(&"Hello", &i.value));

    // This should not lead to reading a moved value or anything
    let owned = data
        .try_map::<DropperRef, _, _>(|_, _| Err("Haha"))
        .unwrap_err()
        .owned;

    assert_eq!("Hello", &*owned);
}

#[test]
pub fn cloned_map() {
    let data = Rc::pin("Hello, world!".to_owned());
    let selfie: Selfie<Rc<String>, Ref<str>> = Selfie::new(data, |str| &str[0..5]);
    selfie.with_referential(|s| assert_eq!("Hello", *s));

    let second_selfie = selfie.map_cloned::<Ref<str>, _>(|str, _| &str[3..]);
    second_selfie.with_referential(|s| assert_eq!("lo", *s));
    selfie.with_referential(|s| assert_eq!("Hello", *s)); // Old one still works

    drop(selfie);
    second_selfie.with_referential(|s| assert_eq!("lo", *s)); // New one still works
}
