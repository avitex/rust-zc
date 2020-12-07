use zc::{Dependant, Zc};

#[derive(Dependant)]
pub struct StructWithNoLifetime;

#[derive(Dependant)]
pub struct StructWithOneLifetime<'a>(&'a ());

#[derive(PartialEq, Debug, Dependant)]
pub struct StructWithBytes<'a>(&'a [u8]);

#[test]
fn test_struct_with_bytes() {
    let owner = vec![1, 2, 3];
    let data = Zc::new(owner, |bytes| StructWithBytes(&bytes[1..]));

    assert_eq!(
        data.dependant::<StructWithBytes>(),
        &StructWithBytes(&[2, 3])
    )
}

#[test]
fn invalid_use() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
}
