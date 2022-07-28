use core::pin::Pin;
use selfie::refs::Ref;
use selfie::Selfie;

// From https://users.rust-lang.org/t/soundness-review-for-selfie-my-personal-self-referential-struct-library/79010/2?u=prokopyl

fn main() {
    let data: Pin<String> = Pin::new("Hello, world!".to_owned());
    let mut selfie: Selfie<String, Ref<str>> = Selfie::new(data, |s| &*s);

    {
        let new_string = String::from("foo");
        selfie.with_referential_mut(|s| *s = &new_string);
    }

    println!("{}", selfie.with_referential(|r| *r));
}
