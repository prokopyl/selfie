#![cfg_attr(not(feature = "std"), no_std)]

pub mod refs;
pub(crate) mod utils;

mod selfie;
pub use crate::selfie::*;

mod pinned_selfie;
pub use pinned_selfie::*;

mod stable_owned;
pub use stable_owned::*;

pub(crate) mod unsafe_selfie;
