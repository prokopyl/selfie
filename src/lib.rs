#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)] // Unsafe code will be allowed in specific modules on a case-by-case basis

pub mod refs;
pub(crate) mod utils;

mod selfie;
pub use crate::selfie::*;

mod safe;

pub mod convert;

mod pinned_selfie;
pub use pinned_selfie::*;

mod stable_owned;
pub use stable_owned::*;
