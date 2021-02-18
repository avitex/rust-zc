//! Crate providing [`Zc`] for self-referential zero-copy structures.

#![no_std]
#![forbid(
    clippy::pedantic,
    rust_2018_idioms,
    anonymous_parameters,
    unused_qualifications,
    //missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    warnings
)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod r#impl;
// FIXME: Remove the need for macros.
mod macros;
mod private;

use core::fmt::{self, Debug, Display};
use core::ops::Deref;

#[cfg(feature = "alloc")]
pub use aliasable;

#[cfg(feature = "derive")]
pub use zc_derive::Dependant;

use self::private::{Construct, TryConstruct};

/// Zero-copy structure consisting of an [`Owner`] and a [`Dependant`].
pub struct Zc<O: Owner, D> {
    // SAFETY: Order of fields is important for preventing dropping the storage
    // before the value that references it.
    value: D,
    storage: O::Storage,
}

impl<O, D> Zc<O, D>
where
    O: Owner,
    D: Dependant<'static>,
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
    ///     data.get::<MyStruct>(),
    ///     &MyStruct(&[2, 3])
    /// );
    /// ```
    // FIXME: This interface isn't the nicest as you have to specific the
    // dependant again to retrieve it. GATs should provide us a way to make this
    // nicer with a generic associated lifetime.
    // See: https://github.com/rust-lang/rust/issues/44265
    pub fn get<'a, T>(&'a self) -> &T
    where
        T: Dependant<'a, Static = D>,
    {
        let value_ptr: *const D = &self.value;
        unsafe { &*value_ptr.cast::<T>() }
    }
}

impl<O, D> Zc<O, D>
where
    O: Owner,
{
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
    /// assert_eq!(data.as_owned(), &[1, 2, 3]);
    /// ```
    pub fn as_owned(&self) -> &<O::Storage as Deref>::Target {
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

    /// Map the stored [`Dependant`] to another.
    ///
    /// # Safety
    ///
    /// The [`Dependant`] passed to the function has its lifetime erased to
    /// `'static` and must be handled appropriately. Nothing within the
    /// [`Dependant`] passed can be referenced from outside of the closure.
    #[inline]
    pub unsafe fn map_unchecked<F, U>(self, f: F) -> Zc<O, U>
    where
        F: FnOnce(D) -> U,
    {
        let Self { value, storage } = self;
        let value = f(value);
        Zc { value, storage }
    }

    /// Try to map the stored [`Dependant`] to another.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function returns.
    ///
    /// # Safety
    ///
    /// The [`Dependant`] passed to the function has its lifetime erased to
    /// `'static` and must be handled appropriately. Nothing within the
    /// [`Dependant`] passed can be referenced from outside of the closure,
    /// this includes the error returned.
    #[inline]
    pub unsafe fn try_map_unchecked<F, U, E>(self, f: F) -> Result<Zc<O, U>, E>
    where
        F: FnOnce(D) -> Result<U, E>,
    {
        let Self { value, storage } = self;
        f(value).map(|value| Zc { value, storage })
    }
}

impl<O, T> Zc<O, Option<T>>
where
    O: Owner,
{
    /// Decomposes `self` into an option.
    #[inline]
    pub fn into_option(self) -> Option<Zc<O, T>> {
        self.into()
    }
}

impl<O, Ok, Err> Zc<O, Result<Ok, Err>>
where
    O: Owner,
{
    /// Decomposes `self` into a result.
    ///
    /// # Errors
    ///
    /// Returns `Zc<O, Err>` if the inner dependant is `Result::Err`.
    #[inline]
    pub fn into_result(self) -> Result<Zc<O, Ok>, Zc<O, Err>> {
        self.into()
    }
}

impl<O, T> From<Zc<O, Option<T>>> for Option<Zc<O, T>>
where
    O: Owner,
{
    #[inline]
    fn from(zc: Zc<O, Option<T>>) -> Self {
        match zc.value {
            None => None,
            Some(value) => Some(Zc {
                value,
                storage: zc.storage,
            }),
        }
    }
}

impl<O, Ok, Err> From<Zc<O, Result<Ok, Err>>> for Result<Zc<O, Ok>, Zc<O, Err>>
where
    O: Owner,
{
    #[inline]
    fn from(zc: Zc<O, Result<Ok, Err>>) -> Self {
        match zc.value {
            Ok(value) => Ok(Zc {
                value,
                storage: zc.storage,
            }),
            Err(value) => Err(Zc {
                value,
                storage: zc.storage,
            }),
        }
    }
}

impl<O, D> Display for Zc<O, D>
where
    O: Owner,
    D: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl<O, D> Debug for Zc<O, D>
where
    O: Owner,
    O::Storage: Debug,
    D: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zc")
            .field("storage", &self.storage)
            .field("value", &self.value)
            .finish()
    }
}

/// Implemented for types that use data provided by an [`Owner`] and guarantee
/// that internal state is protected.
///
/// # Derive implementations (recommended)
///
/// It is recommended not to implement this manually and instead use the
/// provided proc-macro as show below.
///
/// ```
/// use zc::Dependant;
///
/// #[derive(Dependant)]
/// pub struct MyStruct<'a> {
///     value: &'a str,
/// }
/// ```
///
/// # Derive implementations for `Copy`
///
/// If a type implements [`Copy`] it cannot support interior mutability and
/// therefore is a valid `Dependant` type.
///
/// To use a [`Copy`] type without having to implement `Dependant` you can tell
/// the derive implementation to check based on a [`Copy`] bound for a specific
/// field or all fields.
///
/// ```
/// use zc::Dependant;
///
/// #[derive(Copy, Clone)]
/// pub struct CopyType;
///
/// #[derive(Dependant)]
/// pub struct StructWithCopy<'a> {
///     // This field has a `Copy` bound.
///     #[zc(check = "Copy")]
///     field_a: &'a CopyType,
///     // This field has the standard `Dependant` bound.
///     field_b: u8,
/// }
///
/// // All fields in this struct have the `Copy` bound.
/// #[derive(Dependant)]
/// #[zc(check = "Copy")]
/// pub struct StructWithAllCopy<'a> {
///     field_a: &'a CopyType,
///     field_b: u8,
/// }
/// ```
///
/// # Manual implementations
///
/// If you wish not to use the provided proc-macro you implement as shown:
///
/// ```
/// struct MyStruct<'a>(&'a [u8]);
///
/// unsafe impl<'a> zc::Dependant<'a> for MyStruct<'a> {
///     type Static = MyStruct<'static>;
/// }
/// ```
///
/// # Safety
///
/// Implementer must guarantee:
///
/// - the structure only requires a single lifetime.
/// - `Self::Static` must be the same type but with a `'static` lifetime.
///
/// And in addition the structure:
///
/// - has no interior mutability.
///
/// **OR**
///
/// - can safely be stored with it's lifetime erased (ie. as `'static`).
/// - does not provided an interface that will accept data with non-`'static`
///   lifetime though a interior mutable interface.
///
/// # Interior Mutability
///
/// Types that provide interior mutability include both `!Sync` types (eg.
/// [`RefCell<T>`]) and `Sync` types (eg. [`Mutex<T>`]).
///
/// See the [Rust Language Book] on interior mutability.
///
/// [`Mutex<T>`]: std::sync::Mutex
/// [`RefCell<T>`]: std::cell::RefCell
/// [Rust Language Book]: https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
pub unsafe trait Dependant<'a>: Sized + 'a {
    /// Always the exact same structure as `Self` but instead with a `'static`
    /// lifetime.
    type Static: Dependant<'static>;
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

/// Implemented for types that can safely provide a stable, aliasable reference
/// to data they own.
///
/// # `noalias`
///
/// The pointers behind common allocation types (`Box<T>`, `Vec<T>`, etc), are
/// stored via `core::ptr::Unique<T>`, which passes to the compilier a `noalias`
/// attribute. This attribute allows the compiler to make optimisations with the
/// guarantee that no other pointers are referencing the same data.
///
/// We want to both own the data and provide a reference to it, outside of
/// Rust's normal lifetime guarantees, which can break with some of the
/// optimisations the compiler can make. To achieve this, we need to remove the
/// `noalias` attribute of the underlying pointer to let the compiler know that
/// there will exist multiple pointers referencing the same owned data, which is
/// also known as aliasing.
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
