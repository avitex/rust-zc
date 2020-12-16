use crate::NoInteriorMut;

///////////////////////////////////////////////////////////////////////////////
// NoInteriorMut impl

macro_rules! impl_no_interior_mut {
    ($($ty:ty),*) => {
        $(unsafe impl NoInteriorMut for $ty {})*
    };
}

impl_no_interior_mut!(());
impl_no_interior_mut!(bool, char);
impl_no_interior_mut!(f32, f64);
impl_no_interior_mut!(isize, usize);
impl_no_interior_mut!(u8, u16, u32, u64, u128);
impl_no_interior_mut!(i8, i16, i32, i64, i128);
impl_no_interior_mut!(&str, &[u8]);

unsafe impl<T: NoInteriorMut> NoInteriorMut for &T {}
unsafe impl<T: NoInteriorMut> NoInteriorMut for Option<T> {}
unsafe impl<T: NoInteriorMut, E: NoInteriorMut> NoInteriorMut for Result<T, E> {}

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

    use crate::{NoInteriorMut, Owner, Storage};

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
    // NoInteriorMut impl

    unsafe impl NoInteriorMut for String {}
    unsafe impl<T: NoInteriorMut> NoInteriorMut for Vec<T> {}
    unsafe impl<T: NoInteriorMut> NoInteriorMut for BTreeSet<T> {}
    unsafe impl<T: NoInteriorMut> NoInteriorMut for BinaryHeap<T> {}
    unsafe impl<K: NoInteriorMut, V: NoInteriorMut> NoInteriorMut for BTreeMap<K, V> {}
}

///////////////////////////////////////////////////////////////////////////////
// std

#[cfg(feature = "std")]
mod std {
    use std::collections::{HashMap, HashSet};

    use crate::NoInteriorMut;

    ///////////////////////////////////////////////////////////////////////////
    // NoInteriorMut impl

    unsafe impl<T: NoInteriorMut> NoInteriorMut for HashSet<T> {}
    unsafe impl<K: NoInteriorMut, V: NoInteriorMut> NoInteriorMut for HashMap<K, V> {}
}

///////////////////////////////////////////////////////////////////////////////
// NoInteriorMut impl for tuples and arrays

macro_rules! impl_no_interior_mut_tuple {
    ($($name:ident)+) => {
        unsafe impl< $($name: NoInteriorMut),+ > NoInteriorMut for ($($name,)+) {}
    }
}

// FIXME: Replace with const-generics
macro_rules! impl_no_interior_mut_array {
    ($($n:literal)+) => {
        $(unsafe impl<T: NoInteriorMut> NoInteriorMut for [T; $n] {})*
    }
}

impl_no_interior_mut_tuple!(T1);
impl_no_interior_mut_tuple!(T1 T2);
impl_no_interior_mut_tuple!(T1 T2 T3);
impl_no_interior_mut_tuple!(T1 T2 T3 T4);
impl_no_interior_mut_tuple!(T1 T2 T3 T4 T5);
impl_no_interior_mut_tuple!(T1 T2 T3 T4 T5 T6);
impl_no_interior_mut_tuple!(T1 T2 T3 T4 T5 T6 T7);
impl_no_interior_mut_tuple!(T1 T2 T3 T4 T5 T6 T7 T8);

impl_no_interior_mut_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32);
