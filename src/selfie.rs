//! This internal module contains the implementation details for Selfie and SelfieMut.
//!
//! **Do not make any change here without adding new regression, compile-fail and/or MIRI tests!**
//!
//! I do not trust myself in here, and neither should you.

#![allow(unsafe_code)] // I'll be glad to remove this the day self-referential structs can be implemented in Safe Rust

use crate::refs::*;
use crate::utils::*;
use crate::SelfieError;
use core::ops::DerefMut;
use core::pin::Pin;
use stable_deref_trait::{CloneStableDeref, StableDeref};

/// A self-referential struct with a shared reference (`R`) to an object owned by a pinned pointer (`P`).
///
/// If you need a self-referential struct with an exclusive (mutable) reference to the data behind `P`, see [`SelfieMut`].
///
/// This struct is a simple wrapper containing both the pinned pointer `P` and the shared reference to it `R` alongside it.
/// It does not perform any additional kind of boxing or allocation.
///
/// A [`Selfie`] is constructed by using the [`new`](Selfie::new) constructor, which requires the pinned pointer `P`,
/// and a function to create the reference type `R` from a shared reference to the data behind `P`.
///
/// Because `R` references the data behind `P` for as long as this struct exists, the data behind `P`
/// has to be considered to be borrowed for the lifetime of the [`Selfie`].
///
/// Therefore, you can only access the data behind `P` through shared references (`&T`) using [`owned`](Selfie::owned), or by
/// using [`into_owned`](Selfie::into_owned), which drops `R` and returns `P` as it was given to
/// the constructor.
///
/// Note that the referential type `R` is not accessible outside of the [`Selfie`] either, and can
/// only be accessed by temporarily borrowing it through the [`with_referential`](Selfie::with_referential)
/// and [`with_referential_mut`](Selfie::with_referential_mut) methods, which hide its true lifetime.
///
/// This is done because `R` actually has a self-referential lifetime, which cannot be named
/// in Rust's current lifetime system.
///
/// Also because of the non-nameable self-referential lifetime, `R` is not the referential type
/// itself, but a stand-in that implements [`RefType`] (e.g. [`Ref<T>`](Ref) instead of `&T`).
/// See the [`refs`](crate::refs) module for some reference type stand-ins this library provides, or see
/// the [`RefType`] trait documentation for how to implement your own.
///
/// # Example
///
/// This example stores both an owned `String` and a [`str`] slice pointing
/// into it.
///
/// ```
/// use core::pin::Pin;
/// use selfie::{refs::Ref, Selfie};
///
/// let data: Pin<String> = Pin::new("Hello, world!".to_owned());
/// let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |s| &s[0..5]);
///
/// assert_eq!("Hello", selfie.with_referential(|r| *r));
/// assert_eq!("Hello, world!", selfie.owned());
/// ```
pub struct Selfie<'a, P, R>
where
    P: 'a,
    R: RefType<'a>,
{
    // SAFETY: enforce drop order!
    // SAFETY: Note that Ref's lifetime isn't actually ever 'a: it is the unnameable 'this instead.
    // Marking it as 'a is a trick to be able to store it and still name the whole type.
    // It is *absolutely* unsound to ever use this field as 'a, it should immediately be casted
    // to and from 'this instead.
    referential: R::Ref<'a>,
    owned: Pin<P>,
}

impl<'a, P, R> Selfie<'a, P, R>
where
    P: StableDeref + 'a,
    R: RefType<'a>,
    P::Target: 'a,
{
    /// Creates a new [`Selfie`] from a pinned pointer `P`, and a closure to create the reference
    /// type `R` from a shared reference to the data behind `P`.
    ///
    /// Note the closure cannot expect to be called with a specific lifetime, as it will handle
    /// the unnameable `'this` lifetime instead.
    ///
    /// # Example
    ///
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Ref;
    /// use selfie::Selfie;
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |s| &s[0..5]);
    ///
    /// // The selfie now contains both the String buffer and a subslice to "Hello"
    /// assert_eq!("Hello", selfie.with_referential(|r| *r));
    /// ```
    #[inline]
    pub fn new<F>(owned: Pin<P>, handler: F) -> Self
    where
        F: for<'this> FnOnce(&'this P::Target) -> R::Ref<'this>,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();

        Self {
            referential: handler(detached),
            owned,
        }
    }

    /// Creates a new [`Selfie`] from a pinned pointer `P`, and a fallible closure to create the
    /// reference type `R` from a shared reference to the data behind `P`.
    ///
    /// Note the closure cannot expect to be called with a specific lifetime, as it will handle
    /// the unnameable `'this` lifetime instead.
    ///
    /// # Errors
    ///
    /// The closure can return a [`Result`] containing either the referential type, or any error type.
    /// If the closure returns an `Err`, it will be returned in a [`SelfieError`] alongside the original
    /// owned pointer type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Ref;
    /// use selfie::{Selfie, SelfieError};
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: Result<Selfie<String, Ref<str>>, SelfieError<String, ()>>
    ///     = Selfie::try_new(data, |s| Ok(&s[0..5]));
    ///
    /// assert_eq!("Hello", selfie.unwrap().with_referential(|r| *r));
    /// ```
    #[inline]
    pub fn try_new<E, F>(owned: Pin<P>, handler: F) -> Result<Self, SelfieError<P, E>>
    where
        F: for<'this> FnOnce(&'this P::Target) -> Result<R::Ref<'this>, E>,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();

        let referential = match handler(detached) {
            Ok(r) => r,
            Err(error) => return Err(SelfieError { owned, error }),
        };

        Ok(Self { referential, owned })
    }

    /// Returns a shared reference to the owned type by de-referencing `P`.
    ///
    /// # Example
    ///
    /// ```
    /// use core::pin::Pin;
    /// use selfie::{refs::Ref, Selfie};
    ///
    /// let data: Pin<Box<u32>> = Box::pin(42);
    /// let selfie: Selfie<Box<u32>, Ref<u32>> = Selfie::new(data, |i| i);
    ///
    /// assert_eq!(&42, selfie.owned());
    /// ```
    #[inline]
    pub fn owned(&self) -> &P::Target {
        self.owned.as_ref().get_ref()
    }

    /// Performs an operation borrowing the referential type `R`, and returns its result.
    ///
    /// # Example
    ///
    /// ```
    /// use core::pin::Pin;
    /// use selfie::{refs::Ref, Selfie};
    ///
    /// let data: Pin<Box<u32>> = Box::pin(42);
    /// let selfie: Selfie<Box<u32>, Ref<u32>> = Selfie::new(data, |i| i);
    ///
    /// assert_eq!(50, selfie.with_referential(|r| *r + 8));
    /// ```
    #[inline]
    pub fn with_referential<'s, F, T>(&'s self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'s R::Ref<'this>) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(&self.referential) };
        handler(referential)
    }

    /// Performs an operation mutably borrowing the referential type `R`, and returns its result.
    ///
    /// Note that this operation *cannot* mutably access the data behind `P`, it only mutates the
    /// referential type `R` itself.
    ///
    /// # Example
    ///
    /// ```
    /// use core::pin::Pin;
    /// use selfie::{refs::Ref, Selfie};
    ///
    /// let data: Pin<String> = Pin::new("Hello, world!".to_owned());
    /// let mut selfie: Selfie<String, Ref<str>> = Selfie::new(data, |s| &s[0..5]);
    ///
    /// assert_eq!("Hello", selfie.with_referential(|r| *r));
    /// assert_eq!("Hello, world!", selfie.owned());
    ///
    /// selfie.with_referential_mut(|s| *s = &s[0..2]);
    ///
    /// assert_eq!("He", selfie.with_referential(|r| *r));
    /// assert_eq!("Hello, world!", selfie.owned());
    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'s mut R::Ref<'this>) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_mut::<'s, 'a, R>(&mut self.referential) };
        handler(referential)
    }

    /// Unwraps the [`Selfie`] by dropping the reference type `R`, and returning the owned pointer
    /// type `P`, as it was passed to the constructor.
    ///
    /// # Example
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Ref;
    /// use selfie::Selfie;
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |str| &str[0..5]);
    ///
    /// let original_data: Pin<String> = selfie.into_owned();
    /// assert_eq!("Hello, world!", original_data.as_ref().get_ref());
    /// ```
    #[inline]
    pub fn into_owned(self) -> Pin<P> {
        self.owned
    }

    /// Creates a new [`Selfie`] by consuming this [`Selfie`]'s reference type `R` and producing another
    /// (`R2`), using a given closure.
    ///
    /// The owned pointer type `P` is left unchanged, and a shared reference to the data behind it
    /// is also provided to the closure for convenience.
    ///
    /// This method consumes the [`Selfie`]. If you need to keep it intact, see
    /// [`map_cloned`](Selfie::map_cloned).
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
    /// assert_eq!("Hello", selfie.with_referential(|r| *r));
    ///
    /// let selfie = selfie.map::<Ref<str>, _>(|str, _| &str[3..]);
    /// assert_eq!("lo", selfie.with_referential(|r| *r));
    ///
    /// let selfie: Selfie<String, Ref<str>> = selfie.map(|_, owned| &owned[7..]);
    /// assert_eq!("world!", selfie.with_referential(|r| *r));
    /// ```
    #[inline]
    pub fn map<R2: RefType<'a>, F>(self, mapper: F) -> Selfie<'a, P, R2>
    where
        F: for<'this> FnOnce(R::Ref<'this>, &'this P::Target) -> R2::Ref<'this>,
    {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();
        let referential = mapper(referential, detached);

        Selfie { owned, referential }
    }

    /// Creates a new [`Selfie`] by consuming this [`Selfie`]'s reference type `R` and producing another
    /// (`R2`), using a given fallible closure.
    ///
    /// The owned pointer type `P` is left unchanged, and a shared reference to the data behind it
    /// is also provided to the closure for convenience.
    ///
    /// This method consumes the [`Selfie`]. If you need to keep it intact, see
    /// [`try_map_cloned`](Selfie::try_map_cloned).
    ///
    /// # Errors
    ///
    /// The closure can return a [`Result`] containing either the referential type, or any error type.
    /// If the closure returns an `Err`, it will be returned in a [`SelfieError`] alongside the original
    /// owned pointer type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Ref;
    /// use selfie::{Selfie, SelfieError};
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |str| &str[0..5]);
    /// assert_eq!("Hello", selfie.with_referential(|r| *r));
    ///
    /// let selfie = selfie.try_map::<Ref<str>, (), _>(|str, _| Ok(&str[3..])).unwrap();
    /// assert_eq!("lo", selfie.with_referential(|r| *r));
    ///
    /// let selfie: Result<Selfie<String, Ref<str>>, SelfieError<String,()>> = selfie.try_map(|_, owned| Ok(&owned[7..]));
    /// assert_eq!("world!", selfie.unwrap().with_referential(|r| *r));
    /// ```
    #[inline]
    pub fn try_map<R2: RefType<'a>, E, F>(
        self,
        mapper: F,
    ) -> Result<Selfie<'a, P, R2>, SelfieError<P, E>>
    where
        F: for<'this> FnOnce(R::Ref<'this>, &'this P::Target) -> Result<R2::Ref<'this>, E>,
    {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();
        let referential = match mapper(referential, detached) {
            Ok(r) => r,
            Err(error) => return Err(SelfieError { owned, error }),
        };

        Ok(Selfie { owned, referential })
    }

    /// Creates a new [`Selfie`] by cloning this [`Selfie`]'s reference pointer `P` and producing
    /// a new reference (`R2`), using a given closure.
    ///
    /// The owned pointer type `P` needs to be [`CloneStableDeref`](CloneStableDeref),
    /// as only the pointer itself is going to be cloned, not the data behind it. Both the current
    /// reference `R` and the new `R2` will refer to the data behind `P`.
    ///
    /// This method keeps the original [`Selfie`] unchanged, as only its owned pointer is cloned.
    ///
    /// # Example
    ///
    /// ```
    /// use std::rc::Rc;
    /// use selfie::refs::Ref;
    /// use selfie::Selfie;
    ///
    /// let data = Rc::pin("Hello, world!".to_owned());
    /// let selfie: Selfie<Rc<String>, Ref<str>> = Selfie::new(data, |str| &str[0..5]);
    /// selfie.with_referential(|s| assert_eq!("Hello", *s));
    ///
    /// let second_selfie = selfie.map_cloned::<Ref<str>, _>(|str, _| &str[3..]);
    /// second_selfie.with_referential(|s| assert_eq!("lo", *s));
    /// selfie.with_referential(|s| assert_eq!("Hello", *s)); // Old one still works
    ///
    /// drop(selfie);
    /// second_selfie.with_referential(|s| assert_eq!("lo", *s)); // New one still works
    /// ```
    #[inline]
    pub fn map_cloned<R2: RefType<'a>, F>(&self, mapper: F) -> Selfie<'a, P, R2>
    where
        F: for<'this> FnOnce(&R::Ref<'this>, &'this P::Target) -> R2::Ref<'this>,
        P: CloneStableDeref,
    {
        let owned = self.owned.clone();

        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();
        let referential = mapper(&self.referential, detached);

        Selfie { owned, referential }
    }

    /// Creates a new [`Selfie`] by cloning this [`Selfie`]'s reference pointer `P` and producing
    /// a new reference (`R2`), using a given fallible closure.
    ///
    /// The owned pointer type `P` needs to be [`CloneStableDeref`](CloneStableDeref),
    /// as only the pointer itself is going to be cloned, not the data behind it. Both the current
    /// reference `R` and the new `R2` will refer to the data behind `P`.
    ///
    /// This method keeps the original [`Selfie`] unchanged, as only its owned pointer is cloned.
    ///
    /// # Errors
    ///
    /// The closure can return a [`Result`] containing either the referential type, or any error type.
    /// If the closure returns an `Err`, it will be returned right away.
    ///
    /// # Example
    ///
    /// ```
    /// use std::rc::Rc;
    /// use selfie::refs::Ref;
    /// use selfie::Selfie;
    ///
    /// let data = Rc::pin("Hello, world!".to_owned());
    /// let selfie: Selfie<Rc<String>, Ref<str>> = Selfie::new(data, |str| &str[0..5]);
    /// selfie.with_referential(|s| assert_eq!("Hello", *s));
    ///
    /// let second_selfie = selfie.try_map_cloned::<Ref<str>, (), _>(|str, _| Ok(&str[3..])).unwrap();
    /// second_selfie.with_referential(|s| assert_eq!("lo", *s));
    /// selfie.with_referential(|s| assert_eq!("Hello", *s)); // Old one still works
    ///
    /// drop(selfie);
    /// second_selfie.with_referential(|s| assert_eq!("lo", *s)); // New one still works
    /// ```
    #[inline]
    pub fn try_map_cloned<R2: RefType<'a>, E, F>(&self, mapper: F) -> Result<Selfie<'a, P, R2>, E>
    where
        F: for<'this> FnOnce(&R::Ref<'this>, &'this P::Target) -> Result<R2::Ref<'this>, E>,
        P: CloneStableDeref,
    {
        let owned = self.owned.clone();

        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();
        let referential = mapper(&self.referential, detached)?;

        Ok(Selfie { owned, referential })
    }
}

/// A self-referential struct with a mutable reference (`R`) to an object owned by a pinned pointer (`P`).
///
/// If you only need a self-referential struct with an shared reference to the data behind `P`, see [`Selfie`].
///
/// This struct is a simple wrapper containing both the pinned pointer `P` and the mutable reference to it `R` alongside it.
/// It does not perform any additional kind of boxing or allocation.
///
/// A [`SelfieMut`] is constructed by using the [`new`](SelfieMut::new) constructor, which requires the pinned pointer `P`,
/// and a function to create the reference type `R` from a pinned mutable reference to the data behind `P`.
///
/// Because `R` references the data behind `P` for as long as this struct exists, the data behind `P`
/// has to be considered to be exclusively borrowed for the lifetime of the [`SelfieMut`].
///
/// Therefore, you cannot access the data behind `P` at all, until
/// using [`into_owned`](SelfieMut::into_owned), which drops `R` and returns `P` as it was given to
/// the constructor.
///
/// Note that the referential type `R` is not accessible outside of the [`Selfie`] either, and can
/// only be accessed by temporarily borrowing it through the [`with_referential`](SelfieMut::with_referential)
/// and [`with_referential_mut`](SelfieMut::with_referential_mut) methods, which hide its true lifetime.
///
/// This is done because `R` actually has a self-referential lifetime, which cannot be named
/// in Rust's current lifetime system.
///
/// Also because of the non-nameable self-referential lifetime, `R` is not the referential type
/// itself, but a stand-in that implements [`RefType`] (e.g. [`Ref<T>`](Ref) instead of `&T`).
/// See the [`refs`](crate::refs) module for some reference type stand-ins this library provides, or see
/// the [`RefType`] trait documentation for how to implement your own.
///
/// # Example
///
/// This example stores both an owned `String` and a [`str`] slice pointing
/// into it.
///
/// ```
/// use core::pin::Pin;
/// use selfie::{refs::Ref, Selfie};
///
/// let data: Pin<String> = Pin::new("Hello, world!".to_owned());
/// let selfie: Selfie<String, Ref<str>> = Selfie::new(data, |s| &s[0..5]);
///
/// assert_eq!("Hello", selfie.with_referential(|r| *r));
/// assert_eq!("Hello, world!", selfie.owned());
/// ```
pub struct SelfieMut<'a, P, R>
where
    P: 'a,
    R: RefType<'a>,
{
    // SAFETY: enforce drop order!
    referential: R::Ref<'a>,
    owned: Pin<P>,
}

impl<'a, P, R> SelfieMut<'a, P, R>
where
    P: StableDeref + DerefMut + 'a,
    R: RefType<'a>,
{
    /// Creates a new [`SelfieMut`] from a pinned pointer `P`, and a closure to create the reference
    /// type `R` from a pinned, exclusive reference to the data behind `P`.
    ///
    /// Note the closure cannot expect to be called with a specific lifetime, as it will handle
    /// the unnameable `'this` lifetime instead.
    ///
    /// # Example
    ///
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Mut;
    /// use selfie::SelfieMut;
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: SelfieMut<String, Mut<str>> = SelfieMut::new(data, |s| &mut Pin::into_inner(s)[0..5]);
    ///
    /// // The selfie now contains both the String buffer and a subslice to "Hello"
    /// selfie.with_referential(|r| assert_eq!("Hello", *r));
    /// ```
    pub fn new<F>(mut owned: Pin<P>, handler: F) -> Self
    where
        F: for<'this> FnOnce(Pin<&'this mut P::Target>) -> R::Ref<'this>,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime_mut(owned.as_mut()) };

        Self {
            referential: handler(detached),
            owned,
        }
    }

    /// Creates a new [`SelfieMut`] from a pinned pointer `P`, and a fallible closure to create the
    /// reference type `R` from a pinned, exclusive reference to the data behind `P`.
    ///
    /// Note the closure cannot expect to be called with a specific lifetime, as it will handle
    /// the unnameable `'this` lifetime instead.
    ///
    /// # Errors
    ///
    /// The closure can return a [`Result`] containing either the referential type, or any error type.
    /// If the closure returns an `Err`, it will be returned in a [`SelfieError`] alongside the original
    /// owned pointer type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Mut;
    /// use selfie::{SelfieError, SelfieMut};
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: Result<SelfieMut<String, Mut<str>>, SelfieError<String, ()>> =
    ///     SelfieMut::try_new(data, |s| Ok(&mut Pin::into_inner(s)[0..5]));
    ///
    /// selfie.unwrap().with_referential(|r| assert_eq!("Hello", *r));
    /// ```
    #[inline]
    pub fn try_new<E, F>(mut owned: Pin<P>, handler: F) -> Result<Self, SelfieError<P, E>>
    where
        F: for<'this> FnOnce(Pin<&'this mut P::Target>) -> Result<R::Ref<'this>, E>,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime_mut(owned.as_mut()) };

        let referential = match handler(detached) {
            Ok(r) => r,
            Err(error) => return Err(SelfieError { owned, error }),
        };

        Ok(Self { referential, owned })
    }

    /// Performs an operation borrowing the referential type `R`, and returns its result.
    ///
    /// # Example
    ///
    /// ```
    /// use core::pin::Pin;
    /// use selfie::{refs::Mut, SelfieMut};
    ///
    /// let data: Pin<Box<u32>> = Box::pin(42);
    /// let selfie: SelfieMut<Box<u32>, Mut<u32>> = SelfieMut::new(data, |i| Pin::into_inner(i));
    ///
    /// assert_eq!(50, selfie.with_referential(|r| **r + 8));
    /// ```
    #[inline]
    pub fn with_referential<'s, F, T>(&'s self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'s R::Ref<'this>) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(&self.referential) };
        handler(referential)
    }

    /// Performs an operation mutably borrowing the referential type `R`, and returns its result.
    ///
    /// Note that this operation *can* mutably access the data behind `P`.
    ///
    /// # Example
    ///
    /// ```
    /// use core::pin::Pin;
    /// use selfie::{refs::Mut, SelfieMut};
    ///
    /// let data: Pin<String> = Pin::new("Hello, world!".to_owned());
    /// let mut selfie: SelfieMut<String, Mut<str>> = SelfieMut::new(data, |s| &mut Pin::into_inner(s)[0..5]);
    ///
    /// selfie.with_referential_mut(|s| s.make_ascii_uppercase());
    /// selfie.with_referential(|s| assert_eq!("HELLO", *s));
    ///
    /// let data = Pin::into_inner(selfie.into_owned());
    /// assert_eq!("HELLO, world!", &data);
    /// ```
    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'s mut R::Ref<'this>) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_mut::<'s, 'a, R>(&mut self.referential) };
        handler(referential)
    }

    /// Unwraps the [`SelfieMut`] by dropping the reference type `R`, and returning the owned pointer
    /// type `P`, as it was passed to the constructor.
    ///
    /// # Example
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Mut;
    /// use selfie::SelfieMut;
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: SelfieMut<String, Mut<str>> = SelfieMut::new(data, |str| &mut Pin::into_inner(str)[0..5]);
    ///
    /// let original_data: Pin<String> = selfie.into_owned();
    /// assert_eq!("Hello, world!", original_data.as_ref().get_ref());
    /// ```
    #[inline]
    pub fn into_owned(self) -> Pin<P> {
        self.owned
    }

    /// Creates a new [`SelfieMut`] by consuming this [`SelfieMut`]'s reference type `R` and producing another
    /// (`R2`), using a given closure.
    ///
    /// The owned pointer type `P` is left unchanged.
    ///
    /// This method consumes the [`SelfieMut`].
    ///
    /// # Example
    ///
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Mut;
    /// use selfie::SelfieMut;
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: SelfieMut<String, Mut<str>> = SelfieMut::new(data, |str| &mut Pin::into_inner(str)[0..5]);
    /// selfie.with_referential(|s| assert_eq!("Hello", *s));
    ///
    /// let selfie = selfie.map::<Mut<str>, _>(|str, _| &mut str[3..]);
    /// selfie.with_referential(|s| assert_eq!("lo", *s));
    /// ```
    #[inline]
    pub fn map<R2: RefType<'a>, F>(self, mapper: F) -> Selfie<'a, P, R2>
    where
        F: for<'this> FnOnce(
            R::Ref<'this>,
            &'this (), // This is needed to constrain the lifetime TODO: find a way to remove this
        ) -> R2::Ref<'this>,
    {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        let referential = mapper(referential, &());

        Selfie { owned, referential }
    }

    /// Creates a new [`SelfieMut`] by consuming this [`SelfieMut`]'s reference type `R` and producing another
    /// (`R2`), using a given fallible closure.
    ///
    /// The owned pointer type `P` is left unchanged.
    ///
    /// This method consumes the [`SelfieMut`].
    ///
    /// # Errors
    ///
    /// The closure can return a [`Result`] containing either the referential type, or any error type.
    /// If the closure returns an `Err`, it will be returned in a [`SelfieError`] alongside the original
    /// owned pointer type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::pin::Pin;
    /// use selfie::refs::Mut;
    /// use selfie::SelfieMut;
    ///
    /// let data = Pin::new("Hello, world!".to_owned());
    /// let selfie: SelfieMut<String, Mut<str>> = SelfieMut::new(data, |str| &mut Pin::into_inner(str)[0..5]);
    /// selfie.with_referential(|s| assert_eq!("Hello", *s));
    ///
    /// let selfie = selfie.try_map::<Mut<str>, (), _>(|str, _| Ok(&mut str[3..])).unwrap();
    /// selfie.with_referential(|s| assert_eq!("lo", *s));
    /// ```
    #[inline]
    pub fn try_map<R2: RefType<'a>, E, F>(
        self,
        mapper: F,
    ) -> Result<Selfie<'a, P, R2>, SelfieError<P, E>>
    where
        F: for<'this> FnOnce(
            R::Ref<'this>,
            &'this (), // This is needed to constrain the lifetime TODO: find a way to remove this
        ) -> Result<R2::Ref<'this>, E>,
    {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        let referential = match mapper(referential, &()) {
            Ok(r) => r,
            Err(error) => return Err(SelfieError { owned, error }),
        };

        Ok(Selfie { owned, referential })
    }
}
