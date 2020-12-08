use zc::Dependant;

#[derive(Dependant, PartialEq, Debug)]
pub struct MyStruct<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for MyStruct<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(&bytes)
    }
}

fn main() {
    let owner = vec![1, 2, 3];
    let data = zc::from!(owner, MyStruct, [u8]);
    let dependant_ref = data.dependant::<MyStruct>();
    core::mem::drop(data);
    assert_eq!(
        dependant_ref,
        &MyStruct(&[1, 2, 3])
    )
}
