use dangerous::{Expected, Reader};
use zc::{Dependant, Zc};

#[derive(Dependant, Debug)]
pub struct ParsedResult<'a>(Result<Vec<&'a str>, Expected<'a>>);

fn main() {
    let buf = Vec::from(&b"thisisatag,thisisanothertag"[..]);

    let parsed: Zc<Vec<u8>, ParsedResult> = Zc::new(buf, |bytes| {
        let input = dangerous::input(bytes);
        let result = input.read_all(parse);
        ParsedResult(result)
    });

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
