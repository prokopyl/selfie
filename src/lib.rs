#![no_std]
#![doc = include_str!("../README.md")]
#![deny(unsafe_code)] // Unsafe code will be allowed in specific modules on a case-by-case basis
#![deny(clippy::all)]
#![deny(missing_docs)]

pub mod refs;
pub(crate) mod utils;

mod selfie;
pub use crate::selfie::*;

mod safe;
