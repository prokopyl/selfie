use selfie::refs::RefType;
use selfie::Selfie;

#[derive(Clone)]
struct StrRef<'a> {
    inner: &'a str,
}

struct StrRefType;

impl<'a> RefType<'a> for StrRefType {
    type Ref = StrRef<'a>;
}

pub fn main() {
    let data = "hi".to_owned();
    let selfie: Selfie<String, StrRefType> = Selfie::new(data, |inner| StrRef { inner });

    let cloned = selfie.with_referential(|r| r.clone());
    drop(selfie); // Drops both data and selfie
    println!("{}", cloned.inner); // Boom
}
