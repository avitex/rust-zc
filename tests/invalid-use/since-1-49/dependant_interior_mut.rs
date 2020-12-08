use std::cell::RefCell;

use zc::Dependant;

#[derive(PartialEq, Debug, Dependant)]
pub struct StructWithBytes<'a>(RefCell<&'a [u8]>);

impl<'a> From<&'a [u8]> for StructWithBytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(RefCell::new(&bytes[1..]))
    }
}

fn main() {
    let owner = vec![1, 2, 3];
    let owner2 = vec![6, 6, 6];
    let data = zc::from!(owner, StructWithBytes, [u8]);
    dbg!(data.dependant::<StructWithBytes>());
    *data.dependant::<StructWithBytes>().0.borrow_mut() = &owner2;
    drop(owner2);
    dbg!(data.dependant::<StructWithBytes>());
}
