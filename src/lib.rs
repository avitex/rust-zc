//! Crate providing [`Zc`] for self-referential zero-copy structures.

#![no_std]
#![forbid(
    clippy::pedantic,
    rust_2018_idioms,
    anonymous_parameters,
    unused_qualifications,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    warnings
)]

use core::ops::Deref;
use core::{fmt, mem};

#[cfg(feature = "alloc")]
pub use aliasable;
pub use zc_derive::Dependant;

use self::private::{Construct, TryConstruct};

/// A convenience macro for constructing a [`Zc`] type via a [`Dependant`]'s
/// [`From`].
///
/// See [`Zc::new()`] for an example.
///
/// This macro creates an intermediate function to annotate the lifetime
/// required for the `Construct` trait as the compiler is not smart enough yet
/// to infer it for us. See issues [22340] and [70263].
///
/// [22340]: https://github.com/rust-lang/rust/issues/22340
/// [70263]: https://github.com/rust-lang/rust/issues/70263
// FIXME: Remove this
#[macro_export]
macro_rules! from {
    ($owner:expr, $dependant:ident, $target:ty) => {{
        fn _new_fn(arg: &$target) -> $dependant<'_> {
            $dependant::from(arg)
        }
        zc::Zc::new($owner, _new_fn)
    }};
}

/// A convenience macro for constructing a [`Zc`] type via a [`Dependant`]'s
/// [`TryFrom`].
///
/// See [`Zc::try_new()`] for an example.
///
/// This macro creates an intermediate function to annotate the lifetime
/// required for the `TryConstruct` trait as the compiler is not smart enough
/// yet to infer it for us. See issues [22340] and [70263].
///
/// [22340]: https://github.com/rust-lang/rust/issues/22340
/// [70263]: https://github.com/rust-lang/rust/issues/70263
/// [`TryFrom`]: core::convert::TryFrom
// FIXME: Remove this
#[macro_export]
macro_rules! try_from {
    ($owner:expr, $dependant:ident, $target:ty) => {{
        fn _new_fn(
            arg: &$target,
        ) -> Result<$dependant<'_>, <$dependant as core::convert::TryFrom<&$target>>::Error>
        {
            <$dependant as core::convert::TryFrom<&$target>>::try_from(arg)
        }
        zc::Zc::try_new($owner, _new_fn)
    }};
}

/// A zero-copy structure consisting of an [`Owner`] and a [`Dependant`].
pub struct Zc<O: Owner, D> {
    // SAFETY: Order of fields is important for preventing dropping the storage
    // before the value that references it.
    value: D,
    storage: O::Storage,
}

impl<O, D> Zc<O, D>
where
    O: Owner,
    D: Dependant<'static> + 'static,
{
    /// Construct a new zero-copied structure given an [`Owner`] and a
    /// function for constructing the [`Dependant`].
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    ///
    /// #[derive(Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// impl<'a> From<&'a [u8]> for MyStruct<'a> {
    ///     fn from(bytes: &'a [u8]) -> Self {
    ///         Self(&bytes[1..])
    ///     }
    /// }
    ///
    /// let owner = vec![1, 2, 3];
    /// let _ = zc::from!(owner, MyStruct, [u8]);
    /// ```
    pub fn new<C>(owner: O, constructor: C) -> Self
    where
        C: for<'o> Construct<'o, <O::Storage as Deref>::Target, Dependant = D>,
    {
        let storage = Owner::into_storage(owner);
        // Create a temporary dependant given the target reference.
        let value = unsafe { constructor.construct(storage.deref()) };
        // Construct the zero-copy structure given the raw parts.
        Self { storage, value }
    }

    /// Try construct a new zero-copied structure given an [`Owner`] and a
    /// function for constructing the [`Dependant`].
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    /// use core::convert::TryFrom;
    ///
    /// #[derive(Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// impl<'a> TryFrom<&'a [u8]> for MyStruct<'a> {
    ///     type Error = ();
    ///
    ///     fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
    ///         Ok(Self(&bytes[1..]))
    ///     }
    /// }
    ///
    /// let owner = vec![1, 2, 3];
    /// let _ = zc::try_from!(owner, MyStruct, [u8]);
    /// ```
    ///
    /// # Errors
    /// Returns `E` if the constructor failed.
    pub fn try_new<C, E>(owner: O, constructor: C) -> Result<Self, (E, O)>
    where
        E: 'static,
        C: for<'o> TryConstruct<'o, <O::Storage as Deref>::Target, Error = E, Dependant = D>,
    {
        let storage = Owner::into_storage(owner);
        // Try create a temporary dependant given the target reference.
        match unsafe { constructor.try_construct(storage.deref()) } {
            Ok(value) => Ok(Self { storage, value }),
            Err(err) => Err((err, Owner::from_storage(storage))),
        }
    }

    /// Return a reference to the [`Dependant`].
    ///
    /// The dependant type `T` must be supplied (eg.
    /// `Self::dependant::<MyStruct>(&self)`).
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    ///
    /// #[derive(Debug, PartialEq, Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// impl<'a> From<&'a [u8]> for MyStruct<'a> {
    ///     fn from(bytes: &'a [u8]) -> Self {
    ///         Self(&bytes[1..])
    ///     }
    /// }
    ///
    /// let owner = vec![1, 2, 3];
    /// let data = zc::from!(owner, MyStruct, [u8]);
    ///
    /// assert_eq!(
    ///     data.dependant::<MyStruct>(),
    ///     &MyStruct(&[2, 3])
    /// );
    /// ```
    // FIXME: This interface isn't the nicest as you have to specific the
    // dependant again to retrieve it. GATs should provide us a way to make this
    // nicer with a generic associated lifetime.
    // See: https://github.com/rust-lang/rust/issues/44265
    pub fn dependant<'a, T>(&'a self) -> &T
    where
        T: Dependant<'a, Static = D>,
    {
        unsafe { mem::transmute(&self.value) }
    }

    /// Return a reference to the data [`Owner`] provides.
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    ///
    /// #[derive(Debug, PartialEq, Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// impl<'a> From<&'a [u8]> for MyStruct<'a> {
    ///     fn from(bytes: &'a [u8]) -> Self {
    ///         Self(&bytes[1..])
    ///     }
    /// }
    ///
    /// let owner = vec![1, 2, 3];
    /// let data = zc::from!(owner, MyStruct, [u8]);
    ///
    /// assert_eq!(data.owned(), &[1, 2, 3]);
    /// ```
    pub fn owned(&self) -> &<O::Storage as Deref>::Target {
        &*self.storage
    }

    /// Consumes `self` into the [`Owner`].
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    ///
    /// #[derive(Debug, PartialEq, Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// impl<'a> From<&'a [u8]> for MyStruct<'a> {
    ///     fn from(bytes: &'a [u8]) -> Self {
    ///         Self(&bytes[1..])
    ///     }
    /// }
    ///
    /// let owner = vec![1, 2, 3];
    /// let data = zc::from!(owner, MyStruct, [u8]);
    ///
    /// assert_eq!(data.into_owner(), vec![1, 2, 3]);
    /// ```
    pub fn into_owner(self) -> O {
        Owner::from_storage(self.storage)
    }
}

impl<O, D> fmt::Debug for Zc<O, D>
where
    O: Owner,
    O::Storage: fmt::Debug,
    D: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zc")
            .field("storage", &self.storage)
            .field("value", &self.value)
            .finish()
    }
}

/// `Dependant` is implemented for types that use data provided by an [`Owner`].
///
/// # Implementation
///
/// Do not implement this manually and instead use the provided proc-macro as show below.
///
/// ```
/// use zc::Dependant;
///
/// #[derive(Dependant)]
/// pub struct MyStruct<'a> {
///     value: &'a str,
/// }
/// ```
pub unsafe trait Dependant<'a>: Sized {
    /// Always the exact same structure as `Self` but instead with a `'static` lifetime.
    type Static: Dependant<'static>;

    #[doc(hidden)]
    unsafe fn transmute_into_static(self) -> Self::Static;
}

/// Represents the owner of data with an associated storage type.
///
/// An `Owner` is a convenience trait that can be implemented without the need
/// of `unsafe` that returns a [`Storage`] that does require an `unsafe`
/// implementation. See the notes on [`Storage`] to see why this it is required.
pub trait Owner: Sized + 'static {
    /// The [`Storage`] type the owner uses.
    type Storage: Storage;

    /// Consumes the `Owner` into the associated [`Storage`] type.
    fn into_storage(self) -> Self::Storage;

    /// Consumes the associated [`Storage`] into the `Owner` type.
    fn from_storage(storage: Self::Storage) -> Self;
}

/// Implemented for types that can safely give a stable, aliasable reference to
/// data they own.
///
/// # `noalias`
///
/// The pointers behind common allocation types (`Box<T>`, `Vec<T>`, etc), are
/// stored via `core::ptr::Unique<T>`, which passes to the compilier a `noalias`
/// attribute. This attribute allows the compiler to make optimisations with the
/// guarantee that no other pointers are referencing the same data.
///
/// We want to both own the data and pass a reference to it which can break with
/// some of the optimisations the compiler can make. To achieve this, we need to
/// remove the `noalias` attribute of the underlying pointer to let the compiler
/// know that there will exist multiple pointers referencing the same owned
/// data, which is also known as aliasing.
///
/// # Safety
///
/// The implementer must guarantee that the reference it provides via [`Deref`]
/// will be **both stable and aliasable** for the lifetime of `self`. Stable in
/// this context meaning that the pointer to the data referenced will not
/// change.
///
/// `Box<T>` provides a stable pointer (the location of the data being pointed
/// to will not change) but is not aliasable (see `noalias` above). Instead we
/// can use the basic wrapper types provided by the [`aliasable`] crate.
pub unsafe trait Storage: Sized + Deref + 'static {}

impl<T> Owner for T
where
    T: Storage,
{
    type Storage = T;

    fn into_storage(self) -> Self::Storage {
        self
    }

    fn from_storage(storage: Self::Storage) -> Self {
        storage
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use aliasable::{
        boxed::AliasableBox,
        string::{AliasableString, UniqueString},
        vec::{AliasableVec, UniqueVec},
    };

    use crate::{Owner, Storage};

    unsafe impl Storage for AliasableString {}
    unsafe impl<T: 'static> Storage for AliasableVec<T> {}
    unsafe impl<T: ?Sized + 'static> Storage for AliasableBox<T> {}

    impl Owner for UniqueString {
        type Storage = AliasableString;

        fn into_storage(self) -> Self::Storage {
            Self::Storage::from_unique(self)
        }

        fn from_storage(storage: Self::Storage) -> Self {
            Self::Storage::into_unique(storage)
        }
    }

    impl<T: 'static> Owner for UniqueVec<T> {
        type Storage = AliasableVec<T>;

        fn into_storage(self) -> Self::Storage {
            Self::Storage::from_unique(self)
        }

        fn from_storage(storage: Self::Storage) -> Self {
            Self::Storage::into_unique(storage)
        }
    }
}

mod private {
    use crate::Dependant;

    pub unsafe trait Construct<'o, O: ?Sized>: Sized {
        type Dependant: Dependant<'static>;

        unsafe fn construct(self, owned: &'o O) -> Self::Dependant;
    }

    unsafe impl<'o, O, D, F> Construct<'o, O> for F
    where
        O: ?Sized + 'o,
        D: Dependant<'o>,
        F: FnOnce(&'o O) -> D + 'static,
    {
        type Dependant = D::Static;

        unsafe fn construct(self, owned: &'o O) -> Self::Dependant {
            (self)(owned).transmute_into_static()
        }
    }

    pub unsafe trait TryConstruct<'o, O: ?Sized>: Sized {
        type Error: 'static;
        type Dependant: Dependant<'static>;

        unsafe fn try_construct(self, owned: &'o O) -> Result<Self::Dependant, Self::Error>;
    }

    unsafe impl<'o, O, D, E, F> TryConstruct<'o, O> for F
    where
        E: 'static,
        O: ?Sized + 'o,
        D: Dependant<'o>,
        F: FnOnce(&'o O) -> Result<D, E> + 'static,
    {
        type Error = E;
        type Dependant = D::Static;

        unsafe fn try_construct(self, owned: &'o O) -> Result<Self::Dependant, Self::Error> {
            (self)(owned).map(|d| d.transmute_into_static())
        }
    }
}
