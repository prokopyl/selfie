//! This internal module contains the implementation details for Selfie and SelfieMut.
//!
//! **Do not make any change here without adding new regression, compile-fail and/or MIRI tests!**
//!
//! I do not trust myself in here, and neither should you.

#![allow(unsafe_code)] // I'll be glad to remove this the day self-referential structs can be implemented in Safe Rust
#![allow(missing_docs)] // I'll be glad to remove this the day self-referential structs can be implemented in Safe Rust

use crate::refs::*;
use crate::utils::*;
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
/// in Rust's current lifetime system. However, the [`referential`](Selfie::referential) method is
/// also provided for convenience, which returns a copy of the referential type if it implements [`Copy`]
/// (which is the case for simple references).
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
/// assert_eq!("Hello", selfie.referential());
/// assert_eq!("Hello, world!", selfie.owned());
/// ```
pub struct Selfie<'a, P, R>
where
    P: 'a,
    R: for<'this> RefType<'this>,
{
    // SAFETY: enforce drop order!
    // SAFETY: Note that Ref's lifetime isn't actually ever 'a: it is the unnameable 'this instead.
    // Marking it as 'a is a trick to be able to store it and still name the whole type.
    // It is *absolutely* unsound to ever use this field as 'a, it should immediately be casted
    // to and from 'this instead.
    referential: <R as RefType<'a>>::Ref,
    owned: Pin<P>,
}

impl<'a, P, R> Selfie<'a, P, R>
where
    P: StableDeref + 'a,
    R: for<'this> RefType<'this>,
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
    /// assert_eq!("Hello", selfie.referential());
    /// ```
    #[inline]
    pub fn new<F>(owned: Pin<P>, handler: F) -> Self
    where
        F: for<'this> FnOnce(&'this P::Target) -> <R as RefType<'this>>::Ref,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();

        Self {
            referential: handler(detached),
            owned,
        }
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
        F: for<'this> FnOnce(&'this <R as RefType<'s>>::Ref) -> T,
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
    /// assert_eq!("Hello", selfie.referential());
    /// assert_eq!("Hello, world!", selfie.owned());
    ///
    /// selfie.with_referential_mut(|s| *s = &s[0..2]);
    ///
    /// assert_eq!("He", selfie.referential());
    /// assert_eq!("Hello, world!", selfie.owned());
    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this mut <R as RefType<'s>>::Ref) -> T,
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
    /// This methods consumes the [`Selfie`]. If you need to keep it intact, see
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
    /// assert_eq!("Hello", selfie.referential());
    ///
    /// let selfie = selfie.map::<Ref<str>, _>(|str, _| &str[3..]);
    /// assert_eq!("lo", selfie.referential());
    ///
    /// let selfie: Selfie<String, Ref<str>> = selfie.map(|_, owned| &owned[7..]);
    /// assert_eq!("world!", selfie.referential());
    /// ```
    #[inline]
    pub fn map<R2: for<'this> RefType<'this>, F>(self, mapper: F) -> Selfie<'a, P, R2>
    where
        F: for<'this> FnOnce(
            <R as RefType<'this>>::Ref,
            &'this P::Target,
        ) -> <R2 as RefType<'this>>::Ref,
    {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();
        let referential = mapper(referential, detached);

        Selfie { owned, referential }
    }

    /// Creates a new [`Selfie`] by cloning this [`Selfie`]'s reference pointer `P` and producing
    /// a new reference (`R2`), using a given closure.
    ///
    /// The owned pointer type `P` needs to be [`CloneStableDeref`](stable_deref_trait::CloneStableDeref),
    /// as only the pointer itself is going to be cloned, not the data behind it. Both the current
    /// reference `R` and the new `R2` will refer to the data behind `P`.
    ///
    /// This methods keeps the original [`Selfie`] unchanged, as only its owned pointer is cloned.
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
    pub fn map_cloned<R2: for<'this> RefType<'this>, F>(&self, mapper: F) -> Selfie<'a, P, R2>
    where
        F: for<'this> FnOnce(
            &<R as RefType<'this>>::Ref,
            &'this P::Target,
        ) -> <R2 as RefType<'this>>::Ref,
        P: CloneStableDeref,
    {
        let owned = self.owned.clone();

        let detached = unsafe { detach_lifetime(owned.as_ref()) }.get_ref();
        let referential = mapper(&self.referential, detached);

        Selfie { owned, referential }
    }
}

pub struct SelfieMut<'a, P, R>
where
    P: 'a,
    R: for<'this> RefType<'this>,
{
    // SAFETY: enforce drop order!
    referential: <R as RefType<'a>>::Ref,
    owned: Pin<P>,
}

impl<'a, P, R> SelfieMut<'a, P, R>
where
    P: StableDeref + DerefMut + 'a,
    R: for<'this> RefType<'this>,
{
    pub fn new<F>(mut owned: Pin<P>, handler: F) -> Self
    where
        F: for<'this> FnOnce(Pin<&'this mut P::Target>) -> <R as RefType<'this>>::Ref,
    {
        // SAFETY: This type does not expose anything that could expose referential longer than owned exists
        let detached = unsafe { detach_lifetime_mut(owned.as_mut()) };

        Self {
            referential: handler(detached),
            owned,
        }
    }

    #[inline]
    pub fn with_referential<'s, F, T>(&'s self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_ref::<'s, 'a, R>(&self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn with_referential_mut<'s, F, T>(&'s mut self, handler: F) -> T
    where
        F: for<'this> FnOnce(&'this mut <R as RefType<'s>>::Ref) -> T,
    {
        // SAFETY: Down-casting is safe here, because Ref is actually 's, not 'a
        let referential = unsafe { downcast_mut::<'s, 'a, R>(&mut self.referential) };
        handler(referential)
    }

    #[inline]
    pub fn into_owned(self) -> Pin<P> {
        self.owned
    }

    #[inline]
    pub fn map<R2: for<'this> RefType<'this>>(
        self,
        mapper: impl for<'this> FnOnce(
            <R as RefType<'this>>::Ref,
            &'this (), // This is needed to constrain the lifetime TODO: find a way to remove this
        ) -> <R2 as RefType<'this>>::Ref,
    ) -> Selfie<'a, P, R2> {
        // SAFETY: here we break the lifetime guarantees: we must be very careful to not drop owned before referential
        let Self { owned, referential } = self;

        let referential = mapper(referential, &());

        Selfie { owned, referential }
    }
}
