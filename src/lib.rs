#![no_std]
#![deny(unsafe_code)] // Unsafe code will be allowed in specific modules on a case-by-case basis
#![doc = include_str!("../README.md")]

pub mod refs;
pub(crate) mod utils;

mod selfie;
pub use crate::selfie::*;

mod safe;
