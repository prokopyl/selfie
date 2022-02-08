use selfie::refs::{Mut, Ref, SelfieRef, SelfieRefMut};
use selfie::{Selfie, SelfieMut};
use std::ops::Deref;
use std::pin::Pin;

#[test]
pub fn cascading() {
    let my_str = Pin::new("Hello, world!".to_owned());
    let data: Selfie<String, SelfieRef<Ref<str>, Ref<str>>> = Selfie::new(my_str, |i| {
        let substr = Pin::new(&i[0..5]);
        Selfie::new(substr, |i| &i[3..])
    });

    assert_eq!("Hello, world!", data.owned());
    assert_eq!("Hello", data.referential().owned());
    assert_eq!(&"lo", data.referential().referential());

    let data = Box::new(data);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!("Hello", data.referential().owned());
    assert_eq!(&"lo", data.referential().referential());
}

#[test]
pub fn more_cascading() {
    let my_str = Pin::new("Hello, world!".to_owned());

    #[allow(clippy::type_complexity)] // Yes, I know, that's the point
    let data: Selfie<String, SelfieRef<Ref<str>, SelfieRef<Ref<str>, Ref<str>>>> =
        Selfie::new(my_str, |i| {
            let substr = Pin::new(&i[0..5]);
            Selfie::new(substr, |i| {
                let substr = Pin::new(&i[1..]);
                Selfie::new(substr, |i| &i[2..])
            })
        });

    assert_eq!("Hello, world!", data.owned());
    assert_eq!("Hello", data.referential().owned());
    assert_eq!("ello", data.referential().referential().owned());
    assert_eq!(&"lo", data.referential().referential().referential());

    let data = Box::new(data);

    assert_eq!("Hello, world!", data.owned());
    assert_eq!("Hello", data.referential().owned());
    assert_eq!("ello", data.referential().referential().owned());
    assert_eq!(&"lo", data.referential().referential().referential());
}

#[test]
pub fn cascading_mut() {
    let my_str = Pin::new(b"Hello, world!".to_vec());

    #[allow(clippy::type_complexity)]
    let mut data: SelfieMut<Vec<u8>, SelfieRefMut<Mut<[u8]>, Mut<[u8]>>> =
        SelfieMut::new(my_str, |i| {
            let substr = Pin::new(&mut Pin::into_inner(i)[0..5]);
            SelfieMut::new(substr, |i| &mut Pin::into_inner(i)[3..])
        });

    assert_eq!(&b"lo", data.referential().referential());
    data.referential_mut().referential_mut()[1] = b'a';
    assert_eq!(&b"la", data.referential().referential());

    let data = Box::new(data);

    assert_eq!(&b"la", data.referential().referential());
    let my_str = data.into_inner();
    assert_eq!(b"Hella, world!", &my_str.deref());
}
