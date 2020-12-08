use zc::{Dependant, Zc};
use once_cell::sync::OnceCell;

static THIEF: OnceCell<&'static [u8]> = OnceCell::new();

#[derive(Dependant)]
pub struct MyStruct<'a>(&'a [u8]);

fn steal_owned_data(bytes: &'static [u8]) -> MyStruct<'static> {
    THIEF.get_or_init(|| bytes);
    MyStruct(bytes)
}

fn main() {
    let owner = vec![1, 2, 3];

    let _ = Zc::new(owner, steal_owned_data);

    // should not work
    assert_eq!(THIEF.get().unwrap(), &[1, 2, 3]);
}
