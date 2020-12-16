use core::convert::TryFrom;

use dangerous::{Fatal, Reader};
use zc::Dependant;

#[derive(Dependant, Debug)]
pub struct ParsedResult<'a>(Vec<&'a str>);

impl<'a> TryFrom<&'a [u8]> for ParsedResult<'a> {
    type Error = Fatal;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let input = dangerous::input(bytes);
        input.read_all(parse).map(Self)
    }
}

fn main() {
    let buf = Vec::from(&b"thisisatag,thisisanothertag"[..]);
    let parsed = zc::try_from!(buf, ParsedResult, [u8]).unwrap();
    dbg!(parsed);
}

fn parse<'i, E>(r: &mut Reader<'i, E>) -> Result<Vec<&'i str>, E>
where
    E: dangerous::Error<'i>,
{
    let mut parts = Vec::new();
    loop {
        let s = r.take_while(|b| b != b',').to_dangerous_str::<E>()?;
        parts.push(s);
        if !r.consume_u8_opt(b',') {
            return Ok(parts);
        }
    }
}
