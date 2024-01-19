#![no_std]
#![doc = include_str!("../README.md")]
#![deny(unsafe_code)] // Unsafe code will be allowed in specific modules on a case-by-case basis
#![deny(clippy::all)]
#![deny(missing_docs)]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod refs;
pub(crate) mod utils;

mod async_util;
mod error;
mod stack_async;
pub use error::*;

mod selfie;
pub use crate::selfie::*;

mod safe;
