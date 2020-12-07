use zc::{Dependant, Zc};

#[derive(Dependant, PartialEq, Debug)]
pub struct MyStruct<'a>(&'a [u8]);

fn main() {
    let owner = vec![1, 2, 3];
    let data = Zc::new(owner, |bytes| MyStruct(&bytes));
    let dependant_ref = data.dependant::<MyStruct>();
    core::mem::drop(data);
    assert_eq!(
        dependant_ref,
        &MyStruct(&[1, 2, 3])
    )
}
