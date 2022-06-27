use selfie::refs::{Mut, Ref, SelfieRef, SelfieRefMut};
use selfie::{Selfie, SelfieMut};
use std::pin::Pin;

#[test]
pub fn cascading() {
    let my_str = Pin::new("Hello, world!".to_owned());
    let selfie: Selfie<String, SelfieRef<Ref<str>, Ref<str>>> = Selfie::new(my_str, |i| {
        let substr = Pin::new(&i[0..5]);
        Selfie::new(substr, |i| &i[3..])
    });

    assert_eq!("Hello, world!", selfie.owned());
    selfie.with_referential(|r1| {
        assert_eq!("Hello", r1.owned());
        assert_eq!("lo", r1.referential());
    });

    // Moving the Selfie has no consequence
    let selfie = Box::new(selfie);

    assert_eq!("Hello, world!", selfie.owned());
    selfie.with_referential(|r1| {
        assert_eq!("Hello", r1.owned());
        assert_eq!("lo", r1.referential());
    });
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
    data.with_referential(|r1| {
        assert_eq!("Hello", r1.owned());
        r1.with_referential(|r2| {
            assert_eq!("ello", r2.owned());
            r2.with_referential(|r3| {
                assert_eq!(&"lo", r3);
            })
        })
    });

    let data = Box::new(data);

    assert_eq!("Hello, world!", data.owned());
    data.with_referential(|r1| {
        assert_eq!("Hello", r1.owned());
        r1.with_referential(|r2| {
            assert_eq!("ello", r2.owned());
            r2.with_referential(|r3| {
                assert_eq!(&"lo", r3);
            })
        })
    });
}

#[test]
pub fn cascading_mut() {
    let my_str = Pin::new(b"Hello, world!".to_vec());

    #[allow(clippy::type_complexity)]
    let mut selfie: SelfieMut<Vec<u8>, SelfieRefMut<Mut<[u8]>, Mut<[u8]>>> =
        SelfieMut::new(my_str, |i| {
            let substr = Pin::new(&mut Pin::into_inner(i)[0..5]);
            SelfieMut::new(substr, |i| &mut Pin::into_inner(i)[3..])
        });

    selfie.with_referential(|r1| r1.with_referential(|r2| assert_eq!(&b"lo", r2)));
    selfie.with_referential_mut(|r1| r1.with_referential_mut(|r2| r2[1] = b'a'));
    selfie.with_referential(|r1| r1.with_referential(|r2| assert_eq!(&b"la", r2)));

    let selfie = Box::new(selfie);
    selfie.with_referential(|r1| r1.with_referential(|r2| assert_eq!(&b"la", r2)));
    let my_str = selfie.into_owned();
    assert_eq!(b"Hella, world!", &my_str[..]);
}
