use zc::{Dependant, Zc};
use once_cell::sync::OnceCell;

static THIEF: OnceCell<&'static [u8]> = OnceCell::new();

#[derive(Dependant)]
pub struct MyStruct<'a>(&'a [u8]);

fn main() {
    let owner = vec![1, 2, 3];

    let _ = Zc::new(owner, |bytes| {
        THIEF.get_or_init(|| bytes);
        MyStruct(bytes)
    });

    // should not work
    assert_eq!(THIEF.get().unwrap(), &[1, 2, 3]);
}
