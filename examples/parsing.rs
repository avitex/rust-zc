use dangerous::{BytesReader, Fatal, Input};
use zc::Zc;

fn main() {
    let buf = Vec::from(&b"thisisatag,thisisanothertag"[..]);
    let parsed = Zc::new(buf, parse_bytes);
    dbg!(parsed.into_result().unwrap());
}

fn parse_bytes(bytes: &[u8]) -> Result<Vec<&str>, ()> {
    dangerous::input(bytes)
        .read_all(parse)
        .map_err(|_: Fatal| ())
}

fn parse<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Vec<&'i str>, E>
where
    E: dangerous::Error<'i>,
{
    let mut parts = Vec::new();
    loop {
        let s = r.take_until_opt(b',').to_dangerous_str::<E>()?;
        parts.push(s);
        if !r.consume_opt(b',') {
            return Ok(parts);
        }
    }
}
