use zc::Dependant;
use core::convert::TryFrom;

#[derive(Dependant, Debug)]
pub struct MyStruct<'a>(&'a [u8]);

impl<'a> TryFrom<&'a [u8]> for MyStruct<'a> {
    type Error = &'a [u8];

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        match bytes[0] {
            0 => Ok(MyStruct(&bytes[..])),
            _ => Err(&bytes[..]),
        }
    }
}

fn main() {
    let owner = vec![1, 2, 3];
    let result = zc::try_from!(owner, MyStruct, [u8]);

    // should not work
    assert_eq!(result.unwrap_err().1, &[1, 2, 3]);
}
