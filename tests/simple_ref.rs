use pinhead::{PinHead, PinHeaded, RefHandle};
use std::ops::Deref;

struct StrRef;

impl<'a> PinHeaded<'a> for StrRef {
    type Ref = &'a str;
}

struct SelfReferential {
    strings: PinHead<Box<String>, StrRef>,
}

fn foo<'a>(str: &'a String) -> <StrRef as PinHeaded<'a>>::Ref {
    &str[0..5]
}

#[test]
pub fn simple_ref() {
    let str = Box::pin("Hello, world!".to_owned());

    let strings = PinHead::<Box<String>, StrRef>::new(str, move |s| &s[0..5]);

    let self_referential = SelfReferential { strings };

    println!(
        "{} <- {}",
        self_referential.strings.owned(),
        self_referential.strings.referential()
    )
}
