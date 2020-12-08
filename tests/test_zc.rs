use zc::{Dependant, Zc};

#[derive(Dependant)]
pub struct StructWithNoLifetime;

#[derive(Dependant)]
pub struct StructWithOneLifetime<'a>(&'a ());

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

    assert_eq!(
        data.dependant::<StructWithBytes>(),
        &StructWithBytes(&[2, 3])
    )
}

#[test]
fn test_struct_with_bytes_from() {
    let owner = vec![1, 2, 3];
    let data = zc::from!(owner, StructWithBytes, [u8]);

    assert_eq!(
        data.dependant::<StructWithBytes>(),
        &StructWithBytes(&[2, 3])
    )
}
