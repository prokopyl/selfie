//! Reference type stand-ins, to be used with [`Selfie`](Selfie) and
//! [`SelfieMut`](SelfieMut) type declarations.
//!
//! These types are stand-ins that allow to name reference types without naming any associated
//! lifetime. The full type can then be reconstructed with an arbitrary lifetime using the
//! [`RefType`] trait.
//!
//! This is necessary for [`Selfie`](Selfie), as it has to work with a self-referential
//! lifetime, which cannot be explicitly named and has to be reconstructed internally.
//! In essence, this is a roundabout way to achieve Higher-Kinded Polymorphism.
//!
//! This module provides stand-ins for common reference types, but you can create your own by
//! implementing the [`RefType`] trait yourself.

use crate::{Selfie, SelfieMut};
use core::marker::PhantomData;
use core::pin::Pin;

/// A trait for reference type stand-ins to be combined with an arbitrary lifetime `'a`, to recreate
/// the full reference type.
///
/// # Example
///
/// Implementing [`RefType`] for a custom referential type, so it can be used in a [`Selfie`]:
///
/// ```
/// use std::pin::Pin;
/// use selfie::refs::RefType;
/// use selfie::Selfie;
///
/// #[derive(Copy, Clone)]
/// struct MyReferentialType<'a>(&'a str);
///
/// struct MyReferentialTypeStandIn;
/// impl<'a> RefType<'a> for MyReferentialTypeStandIn {
///     type Ref = MyReferentialType<'a>;
/// }
///
/// // MyReferentialType can now be used in Selfies!
/// let data = Pin::new("Hello, world!".to_owned());
/// let selfie: Selfie<String, MyReferentialTypeStandIn> = Selfie::new(data, |str| MyReferentialType(&str[0..5]));
///
/// assert_eq!("Hello", selfie.with_referential(|r| *r).0);
/// ```
///
/// Here is a dummy example showing how [`RefType`] stand-ins are used internally:
///
/// ```
/// use selfie::refs::{Ref, RefType};
///
/// // These two type declarations are equivalent
/// const STR_1: &'static str = "Hello, world!";
/// const STR_2: <Ref<str> as RefType<'static>>::Ref = "Hello, world!";
///
/// assert_eq!(STR_1, STR_2);
/// ```
pub trait RefType {
    /// The full reference type that is to be created when combined with the lifetime `'a`.
    type Ref<'a>: 'a + Sized;
}

/// A stand-in for a shared reference `&T`.
///
/// # Example
///
/// ```
/// use std::pin::Pin;
/// use selfie::refs::Ref;
/// use selfie::Selfie;
///
/// let data = Pin::new("Hello, world!".to_owned());
/// let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |str| &str[0..5]);
///
/// assert_eq!("Hello", selfie.with_referential(|r| *r));
/// ```
pub struct Ref<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> RefType for Ref<T> {
    type Ref<'a> = &'a T where T: 'a;
}

/// A stand-in for a mutable reference `&mut T`.
///
/// # Example
///
/// ```
/// use std::pin::Pin;
/// use selfie::refs::Mut;
/// use selfie::SelfieMut;
///
/// let data = Pin::new("Hello, world!".to_owned());
/// let selfie: SelfieMut<String, Mut<str>> =
///     SelfieMut::new(data, |str| &mut Pin::into_inner(str)[0..5]);
///
/// selfie.with_referential(|str| assert_eq!("Hello", *str));
/// ```
pub struct Mut<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> RefType for Mut<T> {
    type Ref<'a> = &'a mut T where T: 'a;
}

/// A stand-in for a [`Selfie`](Selfie) holding a reference type as its owned pointer.
///
/// # Example
///
/// ```
/// use std::pin::Pin;
/// use selfie::refs::{Ref, SelfieRef};
/// use selfie::Selfie;
///
/// let data = Pin::new("Hello, world!".to_owned());
/// let selfie: Selfie<String, SelfieRef<Ref<str>, Ref<str>>> = Selfie::new(data, |str| {
///     let substr = Pin::new(&str[0..5]);
///     Selfie::new(substr, |str| &str[3..])
/// });
///
/// assert_eq!("Hello, world!", selfie.owned());
/// selfie.with_referential(|r| {
///     assert_eq!("Hello", r.owned());
///     assert_eq!("lo", r.with_referential(|r| *r));
/// });
/// ```
pub struct SelfieRef<P, R>(PhantomData<P>, PhantomData<R>)
where
    P: ?Sized,
    R: ?Sized;

impl<P, R> RefType for SelfieRef<P, R>
where
    P: RefType,
    R: RefType,
{
    type Ref<'a> = Selfie<'a, P::Ref<'a>, R>;
}

/// A stand-in for a [`SelfieMut`](SelfieMut) holding a reference type as its owned pointer.
///
/// # Example
///
/// ```
/// use std::pin::Pin;
/// use selfie::refs::{Mut, SelfieRefMut};
/// use selfie::SelfieMut;
///
/// let data = Pin::new("Hello, world!".to_owned());
/// let selfie: SelfieMut<String, SelfieRefMut<Mut<str>, Mut<str>>> = SelfieMut::new(data, |str| {
///     let substr = Pin::new(&mut Pin::into_inner(str)[0..5]);
///     SelfieMut::new(substr, |str| &mut Pin::into_inner(str)[3..])
/// });
///
/// selfie.with_referential(|inner_selfie| {
///     inner_selfie.with_referential(|str| assert_eq!("lo", *str))
/// });
/// ```
pub struct SelfieRefMut<P, R>(PhantomData<P>, PhantomData<R>)
where
    P: ?Sized,
    R: ?Sized;

impl<P, R> RefType for SelfieRefMut<P, R>
where
    P: RefType,
    R: RefType,
{
    type Ref<'a> = SelfieMut<'a, P::Ref<'a>, R>;
}

// Other std types

impl<R: RefType> RefType for Option<R> {
    type Ref<'a> = Option<R::Ref<'a>>;
}

impl<R: RefType> RefType for Pin<R> {
    type Ref<'a> = Pin<R::Ref<'a>>;
}

#[cfg(any(feature = "alloc", feature = "std"))]
mod alloc_impl {
    extern crate alloc;

    use super::*;
    use alloc::boxed::Box;
    use alloc::rc::Rc;
    use alloc::sync::Arc;

    impl<'owned, R: RefType> RefType for Box<R> {
        type Ref<'a> = Box<R::Ref<'a>>;
    }

    impl<'owned, R: RefType> RefType for Rc<R> {
        type Ref<'a> = Rc<R::Ref<'a>>;
    }

    impl<'owned, R: RefType> RefType for Arc<R> {
        type Ref<'a> = Arc<R::Ref<'a>>;
    }
}
