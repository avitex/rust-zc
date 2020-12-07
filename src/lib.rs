//! Zero-copy structure

#![no_std]
#![forbid(
    clippy::pedantic,
    rust_2018_idioms,
    anonymous_parameters,
    unused_qualifications,
    // missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    warnings
)]

use core::ops::Deref;
use core::pin::Pin;
use core::{fmt, mem};

#[cfg(feature = "alloc")]
pub use aliasable;
pub use zc_derive::Dependant;

/// A zero-copy structure consisting of an [`Owner`] and a [`Dependant`].
pub struct Zc<O: Owner, D> {
    owner: Pin<O::Storage>,
    value: D,
}

impl<O, D> Zc<O, D>
where
    O: Owner,
    D: Dependant<'static> + 'static,
{
    /// Construct a new zero-copied structure given a pinned [`Owner`] and a
    /// function for constructing the [`Dependant`].
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    /// use core::pin::Pin;
    ///
    /// #[derive(Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// let owner = Pin::new(vec![1, 2, 3]);
    /// let _ = Zc::new(owner, |bytes| MyStruct(bytes));
    /// ```
    pub fn new<'t, T, F>(owner: Pin<O>, f: F) -> Self
    where
        F: FnOnce(&'t <O::Storage as Deref>::Target) -> T + 'static,
        T: Dependant<'t, Static = D>,
    {
        let owner = Owner::into_storage(owner);
        // Get a reference to the deref target of the owner.
        let target_ref = owner.as_ref().get_ref();
        // Cast the target reference to a target pointer.
        let target_ptr: *const <O::Storage as Deref>::Target = target_ref;
        // Borrow the target from target ptr for a lifetime of 't.
        //
        // SAFETY: target is only borrowed for 't, which exists only
        // within this scope. `F` is 'static disallowing any non-static
        // references that could be used to break this guarantee.
        let target_ref_reborrowed = unsafe { &*target_ptr };
        // Create a temporary dependant given the target reference.
        let temporary = f(target_ref_reborrowed);
        // Construct the zero-copy structure given the raw parts.
        Self::from_raw_parts(owner, temporary)
    }

    /// Construct a new zero-copied structure given a pinned [`Owner`] and a
    /// function for constructing the [`Dependant`].
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    /// use core::pin::Pin;
    ///
    /// #[derive(Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// let owner = Pin::new(vec![1, 2, 3]);
    /// let _ = Zc::new(owner, |bytes| MyStruct(bytes));
    /// ```
    pub fn try_new<'t, T, E, F>(owner: Pin<O>, f: F) -> Result<Self, (E, Pin<O>)>
    where
        F: FnOnce(&'t <O::Storage as Deref>::Target) -> Result<T, E> + 'static,
        T: Dependant<'t, Static = D>,
    {
        let owner = Owner::into_storage(owner);
        // Get a reference to the deref target of the owner.
        let target_ref = owner.as_ref().get_ref();
        // Cast the target reference to a target pointer.
        let target_ptr: *const <O::Storage as Deref>::Target = target_ref;
        // Borrow the target from target ptr for a lifetime of 't.
        //
        // SAFETY: target is only borrowed for 't, which exists only
        // within this scope. `F` is 'static disallowing any non-static
        // references that could be used to break this guarantee.
        let target_ref_reborrowed = unsafe { &*target_ptr };
        // Try create a temporary dependant given the target reference.
        match f(target_ref_reborrowed) {
            Ok(temporary) => Ok(Self::from_raw_parts(owner, temporary)),
            Err(err) => Err((err, Owner::from_storage(owner))),
        }
    }

    /// Construct a new zero-copied structure given an [`Owner`] and a
    /// function for constructing the [`Dependant`].
    ///
    /// This is a convenience function over `Self::new(..)`.
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    ///
    /// #[derive(Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// let owner = vec![1, 2, 3];
    /// let _ = Zc::pin(owner, |bytes| MyStruct(bytes));
    /// ```
    ///
    /// # Errors
    ///
    pub fn pin<'t, T, F>(owner: O, f: F) -> Self
    where
        O::Target: Unpin,
        F: FnOnce(&'t <O::Storage as Deref>::Target) -> T + 'static,
        T: Dependant<'t, Static = D>,
    {
        Self::new(Pin::new(owner), f)
    }

    /// Try construct a new zero-copied structure given an [`Owner`] and a
    /// function for constructing the [`Dependant`].
    ///
    /// This is a convenience function over `Self::try_new(..)`.
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    ///
    /// #[derive(Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// let owner = vec![1, 2, 3];
    /// let result: Result<_, ((), _)> = Zc::try_pin(owner, |bytes| Ok(MyStruct(bytes)));
    /// assert!(result.is_ok());
    /// ```
    pub fn try_pin<'t, T, E, F>(owner: O, f: F) -> Result<Self, (E, O)>
    where
        O::Target: Unpin,
        F: FnOnce(&'t <O::Storage as Deref>::Target) -> Result<T, E> + 'static,
        T: Dependant<'t, Static = D>,
    {
        Self::try_new(Pin::new(owner), f).map_err(|(err, owner)| (err, Pin::into_inner(owner)))
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
    /// let owner = vec![1, 2, 3];
    /// let data = Zc::pin(owner, |bytes| MyStruct(&bytes[1..]));
    ///
    /// assert_eq!(
    ///     data.dependant::<MyStruct>(),
    ///     &MyStruct(&[2, 3])
    /// );
    /// ```
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
    /// let owner = vec![1, 2, 3];
    /// let data = Zc::pin(owner, |bytes| MyStruct(&bytes[1..]));
    ///
    /// assert_eq!(data.owned(), &[1, 2, 3]);
    /// ```
    pub fn owned(&self) -> &<O::Storage as Deref>::Target {
        &*self.owner
    }

    /// Consumes `self` into the pinned [`Owner`].
    ///
    /// # Example
    /// ```
    /// use zc::{Zc, Dependant};
    /// use core::pin::Pin;
    ///
    /// #[derive(Debug, PartialEq, Dependant)]
    /// struct MyStruct<'a>(&'a [u8]);
    ///
    /// let owner = vec![1, 2, 3];
    /// let data = Zc::pin(owner, |bytes| MyStruct(&bytes[1..]));
    ///
    /// assert_eq!(data.into_owner(), Pin::new(vec![1, 2, 3]));
    /// ```
    pub fn into_owner(self) -> Pin<O> {
        Owner::from_storage(self.owner)
    }

    fn from_raw_parts<'t, T>(owner: Pin<O::Storage>, temporary: T) -> Self
    where
        T: Dependant<'t, Static = D>,
    {
        // Remove the 't lifetime to store within `Zc`.
        //
        // SAFETY: `T` and `D` have the same structure guaranteed by the
        // `Dependant` trait impl. References to the dependant are only
        // safely accessible for public use via `Self::dependant(&self)`,
        // which transmutes the dependant back to a non static lifetime.
        let value = unsafe { temporary.transmute_into_static() };
        // Return the owner and the dependant.
        Self { owner, value }
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
            .field("owner", &self.owner)
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

pub unsafe trait Storage: Sized + Deref + 'static {}

pub unsafe trait Owner: Sized + Deref + 'static {
    type Storage: Storage;

    fn into_storage(this: Pin<Self>) -> Pin<Self::Storage>;

    fn from_storage(storage: Pin<Self::Storage>) -> Pin<Self>;
}

unsafe impl<T> Owner for T
where
    T: Storage,
{
    type Storage = T;

    fn into_storage(this: Pin<Self>) -> Pin<Self::Storage> {
        this
    }

    fn from_storage(storage: Pin<Self::Storage>) -> Pin<Self> {
        storage
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use core::pin::Pin;

    use aliasable::{
        boxed::AliasableBox,
        string::{AliasableString, UniqueString},
        vec::{AliasableVec, UniqueVec},
    };

    use crate::{Owner, Storage};

    unsafe impl Storage for AliasableString {}
    unsafe impl<T: 'static> Storage for AliasableVec<T> {}
    unsafe impl<T: ?Sized + 'static> Storage for AliasableBox<T> {}

    unsafe impl Owner for UniqueString {
        type Storage = AliasableString;

        fn into_storage(this: Pin<Self>) -> Pin<Self::Storage> {
            Self::Storage::from_unique_pin(this)
        }

        fn from_storage(storage: Pin<Self::Storage>) -> Pin<Self> {
            Self::Storage::into_unique_pin(storage)
        }
    }

    unsafe impl<T: 'static> Owner for UniqueVec<T> {
        type Storage = AliasableVec<T>;

        fn into_storage(this: Pin<Self>) -> Pin<Self::Storage> {
            Self::Storage::from_unique_pin(this)
        }

        fn from_storage(storage: Pin<Self::Storage>) -> Pin<Self> {
            Self::Storage::into_unique_pin(storage)
        }
    }
}
