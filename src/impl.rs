use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};

use crate::Dependant;

///////////////////////////////////////////////////////////////////////////////
// Dependant impl

macro_rules! impl_dependant_ref {
    ($($ty:ty),*) => {
        $(
            unsafe impl<'o> Dependant<'o> for &'o $ty {
                type Static = &'static $ty;
            }
        )*
    };
}

macro_rules! impl_dependant {
    ($($ty:ty),*) => {
        $(
            unsafe impl<'o> Dependant<'o> for $ty {
                type Static = $ty;
            }
        )*
    };
}

impl_dependant_ref!(str, [u8]);

impl_dependant!(());
impl_dependant!(bool, char);
impl_dependant!(f32, f64);
impl_dependant!(isize, usize);
impl_dependant!(u8, u16, u32, u64, u128);
impl_dependant!(i8, i16, i32, i64, i128);

impl_dependant!(
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

unsafe impl<'o, T: Dependant<'o>> Dependant<'o> for &'o T {
    type Static = &'static T::Static;
}

unsafe impl<'o, T: Dependant<'o>> Dependant<'o> for Option<T> {
    type Static = Option<T::Static>;
}

unsafe impl<'o, T: Dependant<'o>> Dependant<'o> for Wrapping<T> {
    type Static = Wrapping<T::Static>;
}

unsafe impl<'o, T, E> Dependant<'o> for Result<T, E>
where
    T: Dependant<'o>,
    E: Dependant<'o>,
{
    type Static = Result<T::Static, E::Static>;
}

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

    use crate::{Dependant, Owner, Storage};

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
    // Dependant impl

    impl_dependant!(String);

    unsafe impl<'o, T: Dependant<'o>> Dependant<'o> for Vec<T> {
        type Static = Vec<T::Static>;
    }

    unsafe impl<'o, T: Dependant<'o>> Dependant<'o> for BTreeSet<T> {
        type Static = BTreeSet<T::Static>;
    }

    unsafe impl<'o, T: Dependant<'o>> Dependant<'o> for BinaryHeap<T> {
        type Static = BinaryHeap<T::Static>;
    }

    unsafe impl<'o, K, V> Dependant<'o> for BTreeMap<K, V>
    where
        K: Dependant<'o>,
        V: Dependant<'o>,
    {
        type Static = BTreeMap<K::Static, V::Static>;
    }
}

///////////////////////////////////////////////////////////////////////////////
// std

#[cfg(feature = "std")]
mod std {
    use std::collections::{HashMap, HashSet};
    use std::hash::BuildHasher;

    use crate::Dependant;

    ///////////////////////////////////////////////////////////////////////////
    // Dependant impl

    unsafe impl<'o, T, S> Dependant<'o> for HashSet<T, S>
    where
        T: Dependant<'o>,
        S: BuildHasher + 'static,
    {
        type Static = HashSet<T::Static, S>;
    }

    unsafe impl<'o, K, V, S> Dependant<'o> for HashMap<K, V, S>
    where
        K: Dependant<'o>,
        V: Dependant<'o>,
        S: BuildHasher + 'static,
    {
        type Static = HashMap<K::Static, V::Static, S>;
    }
}

///////////////////////////////////////////////////////////////////////////////
// Dependant impl for tuples and arrays

macro_rules! impl_dependant_tuple {
    ($($name:ident)+) => {
        unsafe impl<'o, $($name: Dependant<'o>),+ > Dependant<'o> for ($($name,)+) {
            type Static = ($($name::Static,)+);
        }
    }
}

// FIXME: Replace with const-generics
macro_rules! impl_dependant_array {
    ($($n:literal)+) => {
        $(unsafe impl<'o, T: Dependant<'o>> Dependant<'o> for [T; $n] {
            type Static = [T::Static; $n];
        })*
    }
}

impl_dependant_tuple!(T1);
impl_dependant_tuple!(T1 T2);
impl_dependant_tuple!(T1 T2 T3);
impl_dependant_tuple!(T1 T2 T3 T4);
impl_dependant_tuple!(T1 T2 T3 T4 T5);
impl_dependant_tuple!(T1 T2 T3 T4 T5 T6);
impl_dependant_tuple!(T1 T2 T3 T4 T5 T6 T7);
impl_dependant_tuple!(T1 T2 T3 T4 T5 T6 T7 T8);

impl_dependant_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32);
