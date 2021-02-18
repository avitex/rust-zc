use zc::aliasable::{boxed::AliasableBox, vec::AliasableVec};
use zc::{Dependant, Zc};

#[derive(Dependant)]
pub struct StructWithNoLifetime;

#[derive(Dependant)]
pub struct ChildType<'a>(&'a ());

#[derive(Dependant)]
pub struct StructWithOneLifetime<'a>(ChildType<'a>);

#[derive(Copy, Clone)]
pub struct CopyType;

#[derive(Dependant)]
#[allow(dead_code)]
pub struct StructWithCopy<'a> {
    #[zc(check = "Copy")]
    field_a: &'a CopyType,
    field_b: (),
}

#[derive(Dependant)]
#[allow(dead_code)]
#[zc(check = "Copy")]
pub struct StructWithAllCopy<'a> {
    field_a: &'a CopyType,
    field_b: (),
}

#[derive(Dependant)]
#[allow(dead_code)]
pub struct StructWithDefault<'a> {
    #[zc(guard = "Default")]
    field: &'a (),
}

#[derive(PartialEq, Debug, Dependant)]
pub struct StructWithBytes<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for StructWithBytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(&bytes[1..])
    }
}

fn construct_struct_with_bytes(bytes: &[u8]) -> StructWithBytes<'_> {
    StructWithBytes(&bytes[1..])
}

#[test]
fn test_struct_with_bytes_construct() {
    let owner = vec![1, 2, 3];
    let data = Zc::new(owner, construct_struct_with_bytes);

    assert_eq!(data.get::<StructWithBytes>(), &StructWithBytes(&[2, 3]));
}

#[test]
fn test_struct_with_bytes_from() {
    let owner = vec![1, 2, 3];
    let data = zc::from!(owner, StructWithBytes, [u8]);

    assert_eq!(data.get::<StructWithBytes>(), &StructWithBytes(&[2, 3]));

    assert_eq!(
        format!("{:?}", data),
        "Zc { storage: [1, 2, 3], value: StructWithBytes([2, 3]) }"
    );
}

#[test]
fn test_struct_with_bytes_try_from() {
    let owner = vec![1, 2, 3];
    let data = zc::try_from!(owner, StructWithBytes, [u8]).unwrap();

    assert_eq!(data.get::<StructWithBytes>(), &StructWithBytes(&[2, 3]));
}

#[test]
fn test_struct_with_str_from() {
    #[derive(PartialEq, Debug, Dependant)]
    pub struct StructWithStr<'a>(&'a str);

    impl<'a> From<&'a str> for StructWithStr<'a> {
        fn from(s: &'a str) -> Self {
            Self(&s[1..])
        }
    }

    let owner = String::from("hello");
    let data = zc::from!(owner, StructWithStr, str);

    assert_eq!(data.get::<StructWithStr>(), &StructWithStr("ello"));
}

#[test]
fn test_struct_with_error() {
    #[derive(PartialEq, Debug, Dependant)]
    pub struct StructWithError<'a>(&'a str);

    impl<'a> core::convert::TryFrom<&'a str> for StructWithError<'a> {
        type Error = ();

        fn try_from(_: &'a str) -> Result<Self, Self::Error> {
            Err(())
        }
    }

    let owner = String::from("hello");
    let result = zc::try_from!(owner, StructWithError, str);

    assert_eq!(result.unwrap_err(), ((), String::from("hello")));
}

#[test]
fn test_aliasable_box() {
    #[derive(PartialEq, Debug, Dependant)]
    pub struct StructWithBoxRef<'a>(&'a u8);

    impl<'a> From<&'a u8> for StructWithBoxRef<'a> {
        fn from(v: &'a u8) -> Self {
            Self(v)
        }
    }

    let owner = AliasableBox::from(Box::new(1u8));
    let data = zc::from!(owner, StructWithBoxRef, u8);

    assert_eq!(data.get::<StructWithBoxRef>(), &StructWithBoxRef(&1));
    assert_eq!(AliasableBox::into_unique(data.into_owner()), Box::new(1));
}

#[test]
fn test_aliasable_vec() {
    #[derive(PartialEq, Debug, Dependant)]
    pub struct StructWithVecRef<'a>(&'a [u8]);

    impl<'a> From<&'a [u8]> for StructWithVecRef<'a> {
        fn from(v: &'a [u8]) -> Self {
            Self(v)
        }
    }

    let owner = AliasableVec::from(vec![1u8]);
    let data = zc::from!(owner, StructWithVecRef, [u8]);

    assert_eq!(data.get::<StructWithVecRef>(), &StructWithVecRef(&[1u8]));
    assert_eq!(AliasableVec::into_unique(data.into_owner()), vec![1]);
}
