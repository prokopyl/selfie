#![cfg_attr(not(feature = "std"), no_std)]

pub mod refs;
pub(crate) mod utils;

mod selfie;
pub use crate::selfie::*;
