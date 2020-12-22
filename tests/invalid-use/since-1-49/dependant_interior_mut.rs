use std::sync::Mutex;

use zc::Dependant;

#[derive(Debug, Dependant)]
pub struct StructWithBytes<'a>(Mutex<&'a [u8]>);

impl<'a> From<&'a [u8]> for StructWithBytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(Mutex::new(&bytes[1..]))
    }
}

fn main() {
    let owner = vec![1, 2, 3];
    let owner2 = vec![6, 6, 6];
    let data = zc::from!(owner, StructWithBytes, [u8]);
    dbg!(data.get::<StructWithBytes>());
    *data.get::<StructWithBytes>().0.lock().unwrap() = &owner2;
    drop(owner2);
    dbg!(data.get::<StructWithBytes>());
}
