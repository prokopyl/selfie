#![no_std]

pub mod refs;
pub(crate) mod utils;

#[cfg(feature = "stable_deref_trait")]
mod selfie;
#[cfg(feature = "stable_deref_trait")]
pub use crate::selfie::*;

mod pinned_selfie;
pub use pinned_selfie::*;
