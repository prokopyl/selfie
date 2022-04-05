# selfie



### _This crate is experimental and not yet ready for production use!_

// TODO

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