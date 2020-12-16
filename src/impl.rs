use crate::NoInteriorMut;

///////////////////////////////////////////////////////////////////////////////
// NoInteriorMut impl

macro_rules! impl_no_interior_mut {
    ($($ty:ty)*) => {
        $(unsafe impl NoInteriorMut for $ty {})*
    };
}

impl_no_interior_mut!(f32 f64);
impl_no_interior_mut!(u8 u16 u32 u64 u128);
impl_no_interior_mut!(i8 i16 i32 i64 i128);
impl_no_interior_mut!(&str & [u8]);

unsafe impl NoInteriorMut for () {}
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
