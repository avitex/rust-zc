use zc::{Dependant, Zc};

#[derive(Dependant, Debug)]
pub struct MyStruct<'a>(&'a [u8]);

fn main() {
    let owner = vec![1, 2, 3];
    let result: Result<_, (&[u8], _)> = Zc::try_new(owner, |bytes| match bytes[0] {
        0 => Ok(MyStruct(&bytes[..])),
        _ => Err(&bytes[..]),
    });

    // should not work
    assert_eq!(result.unwrap_err().1, &[1, 2, 3]);
}
