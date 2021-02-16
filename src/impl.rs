use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};

use crate::Guarded;

///////////////////////////////////////////////////////////////////////////////
// Guarded impl

macro_rules! impl_guarded {
    ($($ty:ty),*) => {
        $(unsafe impl Guarded for $ty {})*
    };
}

impl_guarded!(());
impl_guarded!(bool, char);
impl_guarded!(f32, f64);
impl_guarded!(isize, usize);
impl_guarded!(u8, u16, u32, u64, u128);
impl_guarded!(i8, i16, i32, i64, i128);
impl_guarded!(&str, &[u8]);

impl_guarded!(
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize
);

unsafe impl<T: Guarded> Guarded for &T {}
unsafe impl<T: Guarded> Guarded for Option<T> {}
unsafe impl<T: Guarded> Guarded for Wrapping<T> {}
unsafe impl<T: Guarded, E: Guarded> Guarded for Result<T, E> {}

///////////////////////////////////////////////////////////////////////////////
// alloc

#[cfg(feature = "alloc")]
mod alloc {
    use alloc::{
        collections::{BTreeMap, BTreeSet, BinaryHeap},
        string::String,
        vec::Vec,
    };

    use aliasable::{boxed::AliasableBox, string::AliasableString, vec::AliasableVec};

    use crate::{Guarded, Owner, Storage};

    ///////////////////////////////////////////////////////////////////////////
    // Storage impl

    unsafe impl Storage for AliasableString {}
    unsafe impl<T: 'static> Storage for AliasableVec<T> {}
    unsafe impl<T: ?Sized + 'static> Storage for AliasableBox<T> {}

    ///////////////////////////////////////////////////////////////////////////
    // Owner impl

    impl Owner for String {
        type Storage = AliasableString;

        fn into_storage(self) -> Self::Storage {
            Self::Storage::from_unique(self)
        }

        fn from_storage(storage: Self::Storage) -> Self {
            Self::Storage::into_unique(storage)
        }
    }

    impl<T: 'static> Owner for Vec<T> {
        type Storage = AliasableVec<T>;

        fn into_storage(self) -> Self::Storage {
            Self::Storage::from_unique(self)
        }

        fn from_storage(storage: Self::Storage) -> Self {
            Self::Storage::into_unique(storage)
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Guarded impl

    impl_guarded!(String);

    unsafe impl<T: Guarded> Guarded for Vec<T> {}
    unsafe impl<T: Guarded> Guarded for BTreeSet<T> {}
    unsafe impl<T: Guarded> Guarded for BinaryHeap<T> {}
    unsafe impl<K: Guarded, V: Guarded> Guarded for BTreeMap<K, V> {}
}

///////////////////////////////////////////////////////////////////////////////
// std

#[cfg(feature = "std")]
mod std {
    use std::collections::{HashMap, HashSet};
    use std::hash::BuildHasher;

    use crate::Guarded;

    ///////////////////////////////////////////////////////////////////////////
    // Guarded impl

    unsafe impl<T: Guarded, S: BuildHasher> Guarded for HashSet<T, S> {}
    unsafe impl<K: Guarded, V: Guarded, S: BuildHasher> Guarded for HashMap<K, V, S> {}
}

///////////////////////////////////////////////////////////////////////////////
// Guarded impl for tuples and arrays

macro_rules! impl_guarded_tuple {
    ($($name:ident)+) => {
        unsafe impl< $($name: Guarded),+ > Guarded for ($($name,)+) {}
    }
}

// FIXME: Replace with const-generics
macro_rules! impl_guarded_array {
    ($($n:literal)+) => {
        $(unsafe impl<T: Guarded> Guarded for [T; $n] {})*
    }
}

impl_guarded_tuple!(T1);
impl_guarded_tuple!(T1 T2);
impl_guarded_tuple!(T1 T2 T3);
impl_guarded_tuple!(T1 T2 T3 T4);
impl_guarded_tuple!(T1 T2 T3 T4 T5);
impl_guarded_tuple!(T1 T2 T3 T4 T5 T6);
impl_guarded_tuple!(T1 T2 T3 T4 T5 T6 T7);
impl_guarded_tuple!(T1 T2 T3 T4 T5 T6 T7 T8);

impl_guarded_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32);
