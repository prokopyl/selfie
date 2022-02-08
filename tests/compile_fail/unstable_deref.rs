use core::ops::Deref;
use core::pin::Pin;
use selfie::refs::Ref;
use selfie::Selfie;

struct UnstableInt(pub i32);

impl Deref for UnstableInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn main() {
    let int = Pin::new(UnstableInt(42));
    let data: Selfie<UnstableInt, Ref<i32>> = Selfie::new(int, |i| i);

    assert_eq!(&42, *data.referential());

    let data = Box::new(data);
    assert_eq!(&42, *data.referential());
}