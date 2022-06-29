# Selfie

A lightweight self-referential struct library. Macro-free, allocation-free, and `#![no_std]`.

### _This crate is experimental and not yet ready for production use!_

This crate is an experiment to create simple and allocation-free self-referential structs, which I need in realtime
audio contexts.

While this library is small and extensively tested under MIRI, and I believe it to be safe, it has not yet been
peer-reviewed or seen decent real-world usage.

If you are willing to experiment with it, please do! Soundness and usability reviews and PRs are also more than welcome:
self-referential structs are a complex problem, and the only part I feel confidently smart about is the library's name.

## Advantages

There are other self-referential struct libraries out there, but these didn't quite fit my needs:

* **Macro-free**: `Selfie` does not use neither proc-macros nor macros to make self-referential structs. This allows
  for speedy compile times, but also simple, IDE-friendly syntax, which doesn't get in the way of more complex scenarios.
* **Allocation-free**: Creating a `Selfie` does not perform any allocation, and is in fact a zero-cost operation.
  `Selfie` structs store the owned pointer and the referential type right next to each other, so the
  only indirection is the one from the already existing Owned pointer. This also means the `Selfie` library is entirely `#![no_std]`.
* **Few restrictions for the Owned type**: The only requirement for the Owned type is to be behind a pointer that is
  both Pin-able and stable (i.e. implementing [`StableDeref`](https://docs.rs/stable_deref_trait/latest/stable_deref_trait/trait.StableDeref.html)).
  Candidates from the standard library include `&T`, `&mut T`, `Box`, `Rc`, `Arc`, `String`, `Vec` and others, but any
  pointer provided by an external library such as [basedrop](https://crates.io/crates/basedrop) is also inherently supported.
* **No restrictions for the Referential types**: `Selfie` can be used with any type that has a lifetime relationship
  to the owned type, including any of your custom types (with a bit of boilerplate however, see the examples below).
* **Support for mutable self-references**: `Selfie` also has a `SelfieMut` variant, which allows the referential to be
  constructed with a pinned mutable reference instead of a simple shared reference.
* **Support for non-static pointers**: `Selfie` can be tied to any lifetime, allowing the "Owned" pointer to be also
  borrowing something else.
* **Support for cascading self-references**: Because `Selfie`s can be non-static, and there are no restrictions on
  referential types, a `Selfie` can itself be used as a referential type in another `Selfie`! This allows complex
  self-referential structures to be created.

## Disadvantages

* **Some boilerplate needed**: Although `Selfie` has little inherent complexity, because it doesn't rely on macros, that
  complexity is pushed directly onto the user of this library. This makes the `Selfie` type quite a bit of a mouthful
  at times, and also require some boilerplate to use custom referential types.
* **Referential types can be moved**: This may be an issue if your referential type is referenced by another member of
  your struct, as it will need to have a stable address as well. With the current version of `Selfie` this requires
  multiple pointers (potentially allocating), even though it could be theoretically consolidated in a single allocation.
  This may be addressed by a separate `Selfie` variant in the future.

## Examples

### Caching `String` subslices

```rust
use core::pin::Pin;
use selfie::{refs::Ref, Selfie};

let data: Pin<String> = Pin::new("Hello, world!".to_owned());
let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |s| &s[0..5]);

assert_eq!("Hello", selfie.referential());
assert_eq!("Hello, world!", selfie.owned());
```

### Using custom referential types

```rust
use std::pin::Pin;
use selfie::{refs::RefType, Selfie};

#[derive(Copy, Clone)]
struct MyReferentialType<'a>(&'a str);

struct MyReferentialTypeStandIn;

impl<'a> RefType<'a> for MyReferentialTypeStandIn {
  type Ref = MyReferentialType<'a>;
}

// MyReferentialType can now be used in Selfies!
let data = Pin::new("Hello, world!".to_owned());
let selfie: Selfie<String, MyReferentialTypeStandIn> = Selfie::new(data, |str| MyReferentialType(&str[0..5]));

assert_eq!("Hello", selfie.referential().0);
```

### Mutable self-referential
```rust
use core::pin::Pin;
use selfie::{refs::Mut, SelfieMut};

let data: Pin<String> = Pin::new("Hello, world!".to_owned());
let mut selfie: SelfieMut<String, Mut<str>> = SelfieMut::new(data, |s| &mut Pin::into_inner(s)[0..5]);

selfie.with_referential_mut(|s| s.make_ascii_uppercase());
selfie.with_referential(|s| assert_eq!("HELLO", *s));

// By dropping the referential part, we get back the access to the owned data
let data: String = Pin::into_inner(selfie.into_owned());
assert_eq!("HELLO, world!", &data);
```

### Cascading Selfies

```rust
use std::pin::Pin;
use selfie::refs::{Ref, SelfieRef};
use selfie::Selfie;

let data = Pin::new("Hello, world!".to_owned());
let selfie: Selfie<String, SelfieRef<Ref<str>, Ref<str>>> = Selfie::new(data, |str| {
    let substr = Pin::new(&str[0..5]);
    Selfie::new(substr, |str| &str[3..])
});

assert_eq!("Hello, world!", selfie.owned());
selfie.with_referential(|r| {
    assert_eq!("Hello", r.owned());
    assert_eq!("lo", r.referential());
});
```