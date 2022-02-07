use pinhead::{PinHead, PinHeaded, RefHandle};
use std::pin::Pin;

struct BoxAndRef {
    data: PinHead<Box<i32>, RefHandle<i32>>,
}

#[test]
pub fn simple_ref() {
    let my_int = Box::pin(42);
    let data = PinHead::new(my_int, |i| i);

    let box_and_ref = BoxAndRef { data };
    println!(
        "{} <- {}",
        box_and_ref.data.owned(),
        box_and_ref.data.referential()
    );
}

struct BoxAndStr {
    data: PinHead<Box<str>, RefHandle<str>>,
}

#[test]
pub fn simple_str() {
    let my_int = Pin::new("Hello, world!".to_owned().into_boxed_str());
    let data = PinHead::new(my_int, |i| &i[0..5]);

    let box_and_ref = BoxAndStr { data };
    println!(
        "{} <- {}",
        box_and_ref.data.owned(),
        box_and_ref.data.referential()
    );
}
